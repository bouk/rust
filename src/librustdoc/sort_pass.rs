// Copyright 2012 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! A general sorting pass

use astsrv;
use doc;
use fold::Fold;
use fold;
use pass::Pass;

#[cfg(test)] use extract;

use extra::sort;
use std::clone::Clone;

pub type ItemLtEqOp = @fn(v1: &doc::ItemTag, v2:  &doc::ItemTag) -> bool;

struct ItemLtEq {
    op: ItemLtEqOp,
}

impl Clone for ItemLtEq {
    fn clone(&self) -> ItemLtEq {
        ItemLtEq {
            op: self.op,
        }
    }
}

pub fn mk_pass(name: ~str, lteq: ItemLtEqOp) -> Pass {
    Pass {
        name: name.clone(),
        f: |srv, doc| run(srv, doc, ItemLtEq { op: lteq })
    }
}

fn run(
    _srv: astsrv::Srv,
    doc: doc::Doc,
    lteq: ItemLtEq
) -> doc::Doc {
    let fold = Fold {
        fold_mod: fold_mod,
        .. fold::default_any_fold(lteq)
    };
    (fold.fold_doc)(&fold, doc)
}

fn fold_mod(
    fold: &fold::Fold<ItemLtEq>,
    doc: doc::ModDoc
) -> doc::ModDoc {
    let doc = fold::default_any_fold_mod(fold, doc);
    doc::ModDoc {
        items: sort::merge_sort(doc.items, fold.ctxt.op),
        .. doc
    }
}

#[test]
fn test() {
    fn name_lteq(item1: &doc::ItemTag, item2: &doc::ItemTag) -> bool {
        (*item1).name_() <= (*item2).name_()
    }

    let source = ~"mod z { mod y { } fn x() { } } mod w { }";
    do astsrv::from_str(source) |srv| {
        let doc = extract::from_srv(srv.clone(), ~"");
        let doc = (mk_pass(~"", name_lteq).f)(srv.clone(), doc);
        // hidden __std_macros module at the start.
        assert_eq!(doc.cratemod().mods()[1].name_(), ~"w");
        assert_eq!(doc.cratemod().mods()[2].items[0].name_(), ~"x");
        assert_eq!(doc.cratemod().mods()[2].items[1].name_(), ~"y");
        assert_eq!(doc.cratemod().mods()[2].name_(), ~"z");
    }
}

#[test]
fn should_be_stable() {
    fn always_eq(_item1: &doc::ItemTag, _item2: &doc::ItemTag) -> bool {
        true
    }

    let source = ~"mod a { mod b { } } mod c { mod d { } }";
    do astsrv::from_str(source) |srv| {
        let doc = extract::from_srv(srv.clone(), ~"");
        let doc = (mk_pass(~"", always_eq).f)(srv.clone(), doc);
        // hidden __std_macros module at the start.
        assert_eq!(doc.cratemod().mods()[1].items[0].name_(), ~"b");
        assert_eq!(doc.cratemod().mods()[2].items[0].name_(), ~"d");
        let doc = (mk_pass(~"", always_eq).f)(srv.clone(), doc);
        assert_eq!(doc.cratemod().mods()[1].items[0].name_(), ~"b");
        assert_eq!(doc.cratemod().mods()[2].items[0].name_(), ~"d");
    }
}
