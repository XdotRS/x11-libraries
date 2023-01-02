// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use super::*;
use crate::{element::Element, TsExt};

use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote, ToTokens};

impl Struct {
	pub fn impl_writable(&self, tokens: &mut TokenStream2) {
		let ident = &self.ident;

		// TODO: add generic bounds
		let (impl_generics, type_generics, where_clause) = self.generics.split_for_impl();

		let declare_datasize = if self.content.contains_infer() {
			Some(quote!(let mut datasize: usize = 0;))
		} else {
			None
		};

		let pat = TokenStream2::with_tokens(|tokens| {
			self.content.pat_cons_to_tokens(tokens);
		});

		let writes = TokenStream2::with_tokens(|tokens| {
			for element in &self.content {
				element.write_tokens(tokens, DefinitionType::Basic);

				if self.content.contains_infer() {
					element.add_datasize_tokens(tokens);
				}
			}
		});

		tokens.append_tokens(|| {
			quote!(
				#[automatically_derived]
				impl #impl_generics ::cornflakes::Writable for #ident #type_generics #where_clause {
					fn write_to(
						&self,
						buf: &mut impl ::cornflakes::BufMut,
					) -> Result<(), ::cornflakes::WriteError> {
						#declare_datasize
						// Destructure the struct's fields, if any.
						let Self #pat = self;

						#writes

						Ok(())
					}
				}
			)
		});
	}
}

impl Request {
	pub fn impl_writable(&self, tokens: &mut TokenStream2) {
		let ident = &self.ident;

		// TODO: add generic bounds
		let (impl_generics, type_generics, where_clause) = self.generics.split_for_impl();

		let declare_datasize = if self.content.contains_infer() {
			// The datasize starts at `4` to account for the size of a request's header
			// being 4 bytes.
			Some(quote!(let mut datasize: usize = 4;))
		} else {
			None
		};

		let pat = TokenStream2::with_tokens(|tokens| {
			self.content.pat_cons_to_tokens(tokens);
		});

		let writes = TokenStream2::with_tokens(|tokens| {
			for element in &self.content {
				if !element.is_metabyte() && !element.is_sequence() {
					element.write_tokens(tokens, DefinitionType::Request);

					if self.content.contains_infer() {
						element.add_datasize_tokens(tokens);
					}
				}
			}
		});

		let metabyte = if self.minor_opcode.is_some() {
			quote!(
				buf.put_u8(<Self as ::xrb::Request>::minor_opcode().unwrap());
			)
		} else if let Some(element) = self.content.metabyte_element() {
			TokenStream2::with_tokens(|tokens| {
				element.write_tokens(tokens, DefinitionType::Request);
			})
		} else {
			quote!(
				buf.put_u8(0);
			)
		};

		tokens.append_tokens(|| {
			quote!(
				#[automatically_derived]
				impl #impl_generics ::cornflakes::Writable for #ident #type_generics #where_clause {
					fn write_to(
						&self,
						buf: &mut impl ::cornflakes::BufMut,
					) -> Result<(), ::cornflakes::WriteError> {
						#declare_datasize
						// Destructure the request struct's fields, if any.
						let Self #pat = self;

						// Major opcode
						buf.put_u8(<Self as ::xrb::Request>::major_opcode());
						// Metabyte position
						#metabyte
						// Length
						buf.put_u16(<Self as ::xrb::Request>::length(&self));

						// Other elements
						#writes

						Ok(())
					}
				}
			)
		});
	}
}

impl Reply {
	pub fn impl_writable(&self, tokens: &mut TokenStream2) {
		let ident = &self.ident;

		// TODO: add generic bounds
		let (impl_generics, type_generics, where_clause) = self.generics.split_for_impl();

		let declare_datasize = if self.content.contains_infer() {
			// The datasize starts at `8` to account for the size of a reply's\
			// header being 8 bytes.
			Some(quote!(let mut datasize: usize = 8;))
		} else {
			None
		};

		let pat = TokenStream2::with_tokens(|tokens| {
			self.content.pat_cons_to_tokens(tokens);
		});

		let writes = TokenStream2::with_tokens(|tokens| {
			for element in &self.content {
				if !element.is_metabyte() && !element.is_sequence() {
					element.write_tokens(tokens, DefinitionType::Reply);

					if self.content.contains_infer() {
						element.add_datasize_tokens(tokens);
					}
				}
			}
		});

		let metabyte = if let Some(element) = self.content.metabyte_element() {
			TokenStream2::with_tokens(|tokens| {
				element.write_tokens(tokens, DefinitionType::Reply);
			})
		} else {
			quote!(
				buf.put_u8(0);
			)
		};

		let sequence = match self.content.sequence_element() {
			Some(Element::Field(field)) => &field.formatted,
			_ => panic!("replies must have a sequence field"),
		};

		tokens.append_tokens(|| {
			quote!(
				#[automatically_derived]
				impl #impl_generics ::cornflakes::Writable for #ident #type_generics #where_clause {
					fn write_to(
						&self,
						buf: &mut impl ::cornflakes::BufMut,
					) -> Result<(), ::cornflakes::WriteError> {
						#declare_datasize
						// Destructure the reply struct's fields, if any.
						let Self #pat = self;

						// `1` - indicates this is a reply
						buf.put_u8(1);
						// Metabyte position
						#metabyte
						// Sequence field
						buf.put_u16(#sequence);
						// Length
						buf.put_u32(<Self as ::xrb::Reply>::length(&self));

						// Other elements
						#writes

						Ok(())
					}
				}
			)
		});
	}
}

impl Event {
	pub fn impl_writable(&self, tokens: &mut TokenStream2) {
		let ident = &self.ident;

		// TODO: add generic bounds
		let (impl_generics, type_generics, where_clause) = self.generics.split_for_impl();

		let declare_datasize = if self.content.contains_infer() {
			let datasize: usize = if self.content.sequence_element().is_some() {
				4
			} else {
				1
			};

			Some(quote!(let mut datasize: usize = #datasize;))
		} else {
			None
		};

		let pat = TokenStream2::with_tokens(|tokens| {
			self.content.pat_cons_to_tokens(tokens);
		});

		let writes = TokenStream2::with_tokens(|tokens| {
			for element in &self.content {
				if !element.is_metabyte() && !element.is_sequence() {
					element.write_tokens(tokens, DefinitionType::Event);

					if self.content.contains_infer() {
						element.add_datasize_tokens(tokens);
					}
				}
			}
		});

		let metabyte = if self.content.sequence_element().is_none() {
			None
		} else if let Some(element) = self.content.metabyte_element() {
			Some(TokenStream2::with_tokens(|tokens| {
				element.write_tokens(tokens, DefinitionType::Event);
			}))
		} else {
			Some(quote!(
				buf.put_u8(0);
			))
		};

		let sequence = if let Some(Element::Field(field)) = self.content.sequence_element() {
			let formatted = &field.formatted;

			Some(quote!(buf.put_u16(#formatted);))
		} else {
			None
		};

		tokens.append_tokens(|| {
			quote!(
				#[automatically_derived]
				impl #impl_generics ::cornflakes::Writable for #ident #type_generics #where_clause {
					fn write_to(
						&self,
						buf: &mut impl ::cornflakes::BufMut,
					) -> Result<(), ::cornflakes::WriteError> {
						#declare_datasize
						// Destructure the event struct's fields, if any.
						let Self #pat = self;

						// Event code
						buf.put_u8(<Self as ::xrb::Event>::code());
						// Metabyte position
						#metabyte
						// Sequence field
						#sequence

						// Other elements
						#writes

						Ok(())
					}
				}
			)
		});
	}
}

impl Enum {
	pub fn impl_writable(&self, tokens: &mut TokenStream2) {
		let ident = &self.ident;

		// TODO: add generic bounds
		let (impl_generics, type_generics, where_clause) = self.generics.split_for_impl();

		let discriminants = TokenStream2::with_tokens(|tokens| {
			for variant in &self.variants {
				if let Some((_, expr)) = &variant.discriminant {
					let ident = format_ident!("discrim_{}", variant.ident);

					tokens.append_tokens(|| {
						quote!(
							// Isolate the discriminant's expression in a
							// function so that it doesn't have access to
							// identifiers used in the surrounding generated
							// code.
							#[allow(non_snake_case)]
							fn #ident() -> u8 {
								#expr
							}

							// Call the discriminant's function just once and
							// store it in a variable for later use.
							#[allow(non_snake_case)]
							let #ident = #ident();
						)
					});
				}
			}
		});

		let arms = TokenStream2::with_tokens(|tokens| {
			let mut discrim = quote!(0);

			for variant in &self.variants {
				let ident = &variant.ident;

				let declare_datasize = if variant.content.contains_infer() {
					Some(quote!(let mut datasize: usize = 1;))
				} else {
					None
				};

				if variant.discriminant.is_some() {
					let discrim_ident = format_ident!("discrim_{}", ident);

					discrim = discrim_ident.into_token_stream();
				}

				let pat = TokenStream2::with_tokens(|tokens| {
					variant.content.pat_cons_to_tokens(tokens);
				});

				let writes = TokenStream2::with_tokens(|tokens| {
					for element in &variant.content {
						element.write_tokens(tokens, DefinitionType::Basic);

						if variant.content.contains_infer() {
							element.add_datasize_tokens(tokens);
						}
					}
				});

				tokens.append_tokens(|| {
					quote!(
						Self::#ident #pat => {
							#declare_datasize
							buf.put_u8(#discrim);

							#writes
						},
					)
				});

				quote!(/* discrim */ + 1).to_tokens(&mut discrim);
			}
		});

		tokens.append_tokens(|| {
			quote!(
				#[automatically_derived]
				impl #impl_generics ::cornflakes::Writable for #ident #type_generics #where_clause {
					fn write_to(
						&self,
						buf: &mut impl ::cornflakes::BufMut,
					) -> Result<(), ::cornflakes::WriteError> {
						// Define functions and variables for variants which
						// have custom discriminant expressions.
						#discriminants

						match self {
							#arms
						}

						Ok(())
					}
				}
			)
		});
	}
}
