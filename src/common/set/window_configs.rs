// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::{StackMode, Window};

use bitflags::bitflags;
use xrbk::{
	Buf,
	BufMut,
	ConstantX11Size,
	ReadError,
	ReadResult,
	Readable,
	Writable,
	WriteResult,
	X11Size,
};
use xrbk_macro::{ConstantX11Size, Readable, Writable, X11Size};

bitflags! {
	/// A mask of configured options for a [window].
	///
	/// This mask is used in the [`WindowConfigs` set], as well as in the
	/// [`ConfigureWindowRequest` event].
	///
	/// [window]: Window
	/// [`WindowConfigs` set]: WindowConfigs
	/// [`ConfigureWindowRequest` event]: crate::x11::event::ConfigureWindowRequest
	#[derive(Default, X11Size, Readable, ConstantX11Size, Writable)]
	pub struct WindowConfigMask: u16 {
		/// Whether the [x coordinate] of the [window] is configured.
		///
		/// See [`WindowConfigs::x`] for more information.
		///
		/// [window]: Window
		/// [y coordinate]: WindowConfigs::y
		const X = 0x0001;
		/// Whether the [y coordinate] of the [window] is configured.
		///
		/// See [`WindowConfigs::y`] for more information.
		///
		/// [window]: Window
		/// [y coordinate]: WindowConfigs::y
		const Y = 0x0002;
		/// Whether the [width] of the [window] is configured.
		///
		/// See [`WindowConfigs::width`] for more information.
		///
		/// [window]: Window
		/// [width]: WindowConfigs::width
		const WIDTH = 0x0004;
		/// Whether the [height] of the [window] is configured.
		///
		/// See [`WindowConfigs::height`] for more information.
		///
		/// [window]: Window
		/// [height]: WindowConfigs::height
		const HEIGHT = 0x0008;

		/// Whether the width of the [window]'s border is configured.
		///
		/// See [`WindowConfigs::border_width`] for more information.
		///
		/// [window]: Window
		const BORDER_WIDTH = 0x0010;

		/// Whether a sibling [window] is configured in respect to the
		/// configured [`stack_mode`].
		///
		/// See [`WindowConfigs::sibling`] for more information.
		///
		/// [window]: Window
		/// [`stack_mode`]: WindowConfigs::stack_mode
		const SIBLING = 0x0020;

		/// Whether the [`stack_mode`] of a [window] is configured.
		///
		/// See [`WindowConfigs::stack_mode`] for more information.
		///
		/// [window]: Window
		/// [`stack_mode`]: WindowConfigs::stack_mode
		const STACK_MODE = 0x0040;
	}
}

/// A set of options with which a [window] is configured.
///
/// This set is used in the [`ConfigureWindow` request].
///
/// Unspecified configuration options are taken from the existing geometry of
/// the [window].
///
/// [window]: Window
/// [`ConfigureWindow` request]: crate::x11::request::ConfigureWindow
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct WindowConfigs {
	/// Total [`X11Size`] of these `WindowConfigs`.
	///
	/// This is cached so that it doesn't have to be recalculated each time -
	/// `WindowConfigs` is immutable.
	///
	/// This field is not part of the X11 format for this struct.
	x11_size: usize,

	mask: WindowConfigMask,

	// These represent 16-bit values, but they need to take up four bytes, so the _internal
	// representation only_ is 32 bits.
	x: Option<i32>,
	y: Option<i32>,
	width: Option<u32>,
	height: Option<u32>,

	// As above, this always represents a `u16` value instead.
	border_width: Option<u32>,

	sibling: Option<Window>,

	stack_mode: Option<__StackMode>,
}

impl WindowConfigs {
	/// Returns a new [`WindowConfigsBuilder`] with which a `WindowConfigs` set
	/// can be built.
	#[must_use]
	pub const fn builder() -> WindowConfigsBuilder {
		WindowConfigsBuilder::new()
	}
}

/// A builder used to construct a new [`WindowConfigs` set].
///
/// All configuration options start as [`None`], and can be configured with
/// the other methods on this builder. When the builder is configured,
/// [`build()`] can be used to build the resulting [`WindowConfigs`].
///
/// [`build()`]: WindowConfigsBuilder::build
///
/// [`WindowConfigs` set]: WindowConfigs
#[derive(Clone, Default, Debug, Hash, PartialEq, Eq)]
pub struct WindowConfigsBuilder {
	x11_size: usize,

	mask: WindowConfigMask,

	x: Option<i16>,
	y: Option<i16>,
	width: Option<u16>,
	height: Option<u16>,

	border_width: Option<u16>,

	sibling: Option<Window>,

	stack_mode: Option<StackMode>,
}

impl WindowConfigsBuilder {
	/// Creates a new `WindowConfigsBuilder`.
	///
	/// All configuration options start as [`None`], and can be configured with
	/// the other methods on this builder. When the builder is configured,
	/// [`build()`] can be used to build the resulting [`WindowConfigs`].
	///
	/// [`build()`]: WindowConfigsBuilder::build
	#[must_use]
	pub const fn new() -> Self {
		Self {
			x11_size: WindowConfigMask::X11_SIZE,

			mask: WindowConfigMask::empty(),

			x: None,
			y: None,
			width: None,
			height: None,

			border_width: None,

			sibling: None,

			stack_mode: None,
		}
	}

	/// Builds the resulting [`WindowConfigs` set] with the configured options.
	///
	/// [`WindowConfigs` set]: WindowConfigs
	#[must_use]
	pub fn build(self) -> WindowConfigs {
		WindowConfigs {
			x11_size: self.x11_size,

			mask: self.mask,

			x: self.x.map(Into::into),
			y: self.y.map(Into::into),
			width: self.width.map(Into::into),
			height: self.height.map(Into::into),

			border_width: self.border_width.map(Into::into),

			sibling: self.sibling,

			stack_mode: self.stack_mode.map(__StackMode),
		}
	}

	/// Configures the [x coordinate] of the [window].
	///
	/// See [`WindowConfigs::x`] for more information.
	///
	/// [x coordinate]: WindowConfigs::x
	/// [window]: Window
	pub fn x(&mut self, x: i16) -> &mut Self {
		if self.x.is_none() {
			self.x11_size += 4;
		}

		self.x = Some(x);
		self.mask |= WindowConfigMask::X;

		self
	}
	/// Configures the [y coordinate] of the [window].
	///
	/// See [`WindowConfigs::y`] for more information.
	///
	/// [y coordinate]: WindowConfigs::y
	/// [window]: Window
	pub fn y(&mut self, y: i16) -> &mut Self {
		if self.y.is_none() {
			self.x11_size += 4;
		}

		self.y = Some(y);
		self.mask |= WindowConfigMask::Y;

		self
	}
	/// Configures the [width] of the [window].
	///
	/// See [`WindowConfigs::width`] for more information.
	///
	/// [width]: WindowConfigs::width
	/// [window]: Window
	pub fn width(&mut self, width: u16) -> &mut Self {
		if self.width.is_none() {
			self.x11_size += 4;
		}

		self.width = Some(width);
		self.mask |= WindowConfigMask::WIDTH;

		self
	}
	/// Configures the [height] of the [window].
	///
	/// See [`WindowConfigs::height`] for more information.
	///
	/// [height]: WindowConfigs::height
	/// [window]: Window
	pub fn height(&mut self, height: u16) -> &mut Self {
		if self.height.is_none() {
			self.x11_size += 4;
		}

		self.height = Some(height);
		self.mask |= WindowConfigMask::HEIGHT;

		self
	}

	/// Configures the width of the [window]'s border.
	///
	/// See [`WindowConfigs::border_width`] for more information.
	///
	/// [window]: Window
	pub fn border_width(&mut self, border_width: u16) -> &mut Self {
		if self.border_width.is_none() {
			self.x11_size += 4;
		}

		self.border_width = Some(border_width);
		self.mask |= WindowConfigMask::BORDER_WIDTH;

		self
	}

	/// Configures the sibling [window] which the [`stack_mode`] applies to. If
	/// the sibling is configured, the [`stack_mode`] must be configured too.
	///
	/// See [`WindowConfigs::sibling`] for more information.
	///
	/// # Errors
	/// A [`Match` error] is generated if the sibling is configured without
	/// configuring the [`stack_mode`].
	///
	/// [`Match` error]: crate::x11::error::Match
	/// [window]: Window
	/// [`stack_mode`]: WindowConfigs::stack_mode
	pub fn sibling(&mut self, sibling: Window) -> &mut Self {
		if self.sibling.is_none() {
			self.x11_size += 4;
		}

		self.sibling = Some(sibling);
		self.mask |= WindowConfigMask::SIBLING;

		self
	}

	/// Configures the [window]'s [`stack_mode`].
	///
	/// [window]: Window
	/// [`stack_mode`]: WindowConfigs::stack_mode
	pub fn stack_mode(&mut self, stack_mode: StackMode) -> &mut Self {
		if self.stack_mode.is_none() {
			self.x11_size += 4;
		}

		self.stack_mode = Some(stack_mode);
		self.mask |= WindowConfigMask::STACK_MODE;

		self
	}
}

impl WindowConfigs {
	/// The x coordinate of the [window]'s top-left corner is configured.
	///
	/// [window]: Window
	#[must_use]
	pub fn x(&self) -> Option<i16> {
		self.x.map(|x| {
			x.try_into()
				.expect("must fit into i16; represents i16 value")
		})
	}
	/// The y coordinate of the [window]'s top-left corner is configured.
	///
	/// [window]: Window
	#[must_use]
	pub fn y(&self) -> Option<i16> {
		self.y.map(|y| {
			y.try_into()
				.expect("must fit into i16; represents i16 value")
		})
	}
	/// The width of the [window] is configured.
	///
	/// [window]: Window
	#[must_use]
	pub fn width(&self) -> Option<u16> {
		self.width.map(|width| {
			width
				.try_into()
				.expect("must fit into u16; represents u16 value")
		})
	}
	/// The height of the [window] is configured.
	///
	/// [window]: Window
	#[must_use]
	pub fn height(&self) -> Option<u16> {
		self.height.map(|height| {
			height
				.try_into()
				.expect("must fit into u16; represents u16 value")
		})
	}

	/// The width of the [window]'s border is configured.
	///
	/// [window]: Window
	#[must_use]
	pub fn border_width(&self) -> Option<u16> {
		self.border_width.map(|border_width| {
			border_width
				.try_into()
				.expect("must fit into u16; represents u16 value")
		})
	}

	/// The sibling which the [`stack_mode`] applies to is configured.
	///
	/// # Errors
	/// A [`Match` error] is generated if the sibling is configured without
	/// configuring the [`stack_mode`].
	///
	/// [`stack_mode`]: WindowConfigs::stack_mode
	/// [`Match` error]: crate::x11::error::Match
	#[must_use]
	pub const fn sibling(&self) -> &Option<Window> {
		&self.sibling
	}

	/// The way in which the [window] is stacked compared to its sibling(s) is
	/// configured.
	///
	/// If [`sibling`] is specified, this is relative to that [`sibling`].
	/// Otherwise, this is relative to all other siblings.
	///
	/// [window]: Window
	/// [`sibling`]: WindowConfigs::sibling
	#[must_use]
	pub fn stack_mode(&self) -> Option<&StackMode> {
		self.stack_mode
			.as_ref()
			.map(|__StackMode(stack_mode)| stack_mode)
	}
}

impl X11Size for WindowConfigs {
	fn x11_size(&self) -> usize {
		self.x11_size
	}
}

impl Readable for WindowConfigs {
	fn read_from(buf: &mut impl Buf) -> ReadResult<Self>
	where
		Self: Sized,
	{
		let mask = WindowConfigMask::read_from(buf)?;
		// 2 unused bytes after the mask.
		buf.advance(2);

		let mut x11_size = mask.x11_size() + 2;

		let x = super::read_set_value(buf, &mut x11_size, mask.contains(WindowConfigMask::X))?;
		let y = super::read_set_value(buf, &mut x11_size, mask.contains(WindowConfigMask::Y))?;
		let width =
			super::read_set_value(buf, &mut x11_size, mask.contains(WindowConfigMask::WIDTH))?;
		let height =
			super::read_set_value(buf, &mut x11_size, mask.contains(WindowConfigMask::HEIGHT))?;

		let border_width = super::read_set_value(
			buf,
			&mut x11_size,
			mask.contains(WindowConfigMask::BORDER_WIDTH),
		)?;

		let sibling =
			super::read_set_value(buf, &mut x11_size, mask.contains(WindowConfigMask::SIBLING))?;

		let stack_mode = super::read_set_value(
			buf,
			&mut x11_size,
			mask.contains(WindowConfigMask::STACK_MODE),
		)?;

		Ok(Self {
			x11_size,

			mask,

			x,
			y,
			width,
			height,

			border_width,

			sibling,

			stack_mode,
		})
	}
}

impl Writable for WindowConfigs {
	fn write_to(&self, buf: &mut impl BufMut) -> WriteResult {
		self.mask.write_to(buf)?;
		// 2 unused bytes after the mask.
		buf.put_bytes(0, 2);

		if let Some(x) = self.x {
			x.write_to(buf)?;
		}
		if let Some(y) = self.y {
			y.write_to(buf)?;
		}
		if let Some(width) = self.width {
			width.write_to(buf)?;
		}
		if let Some(height) = self.height {
			height.write_to(buf)?;
		}

		if let Some(border_width) = self.border_width {
			border_width.write_to(buf)?;
		}

		if let Some(sibling) = &self.sibling {
			sibling.write_to(buf)?;
		}

		if let Some(stack_mode) = &self.stack_mode {
			stack_mode.write_to(buf)?;
		}

		Ok(())
	}
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
struct __StackMode(StackMode);

impl ConstantX11Size for __StackMode {
	const X11_SIZE: usize = 4;
}

impl X11Size for __StackMode {
	fn x11_size(&self) -> usize {
		Self::X11_SIZE
	}
}

impl Readable for __StackMode {
	fn read_from(buf: &mut impl Buf) -> ReadResult<Self>
	where
		Self: Sized,
	{
		Ok(Self(match buf.get_u32() {
			discrim if discrim == 0 => StackMode::Above,
			discrim if discrim == 1 => StackMode::Below,
			discrim if discrim == 2 => StackMode::TopIf,
			discrim if discrim == 3 => StackMode::BottomIf,
			discrim if discrim == 4 => StackMode::Opposite,

			other_discrim => {
				return Err(ReadError::UnrecognizedDiscriminant(other_discrim as usize))
			},
		}))
	}
}

impl Writable for __StackMode {
	fn write_to(&self, buf: &mut impl BufMut) -> WriteResult {
		let Self(stack_mode) = self;

		match stack_mode {
			StackMode::Above => buf.put_u32(0),
			StackMode::Below => buf.put_u32(1),
			StackMode::TopIf => buf.put_u32(2),
			StackMode::BottomIf => buf.put_u32(3),
			StackMode::Opposite => buf.put_u32(4),
		}

		Ok(())
	}
}
