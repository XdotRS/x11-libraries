// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

pub struct OuterAttribute;

pub enum Visibility {
	Public(PubVisibility),
	PublicCrate(PubCrateVisibility),
	PublicSelf(PubSelfVisibility),
	PublicSuper(PubSuperVisibility),
	PublicInPath(PubInPathVisibility),

	/// The default visibility; there is no `pub` keyword.
	Default,
}

pub struct PubVisibility;
pub struct PubCrateVisibility;
pub struct PubSelfVisibility;
pub struct PubSuperVisibility;
pub struct PubInPathVisibility;

pub struct AttributesItem {
	pub attributes: Vec<OuterAttribute>,
	pub item: Item,
}

pub enum Item {
	WithVisibility(VisItem),
	Macro(MacroItem),
}

pub struct VisItem {
	pub visibility: Visibility,
	pub definition: VisDefinition,
}

pub enum VisDefinition {
	Module(Module),
	ExternCrate(ExternCrate),
	Use(Use),
	Function(Function),
	TypeAlias(TypeAlias),
	Struct(Struct),
	Enum(Enum),
	Union(Union),
	Constant(Constant),
	Static(Static),
	Trait(Trait),
	Impl(Impl),
	Extern(Extern),
}

pub struct Module;
pub struct ExternCrate;
pub struct Use;
pub struct Function;
pub struct TypeAlias;
pub struct Struct;
pub struct Enum;
pub struct Union;
pub struct Constant;
pub struct Static;
pub struct Trait;
pub struct Impl;
pub struct Extern;

pub enum MacroItem {
	OuterMacroInvocation(OuterMacroInvocation),
	MacroRulesDefinition(MacroRulesDefinition),
}

/// A macro invocation which is treated as an item; the outer `(...)` and
/// `[...]` delimiters have semicolons following them.
pub struct OuterMacroInvocation;
pub struct MacroRulesDefinition;
