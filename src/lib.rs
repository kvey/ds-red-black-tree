#![feature(ptr_internals)]
#![feature(shared)]

// http://cglab.ca/~abeinges/blah/too-many-lists/book/
// reference http://cglab.ca/~abeinges/blah/rust-btree-case/
// https://www.reddit.com/r/rust/comments/3svacd/memory_corruption_of_a_raw_pointer/?st=je8oull8&sh=d9f97f78

use std::boxed::Box;
use std::ptr::NonNull;
use std::ptr;
use std::mem;
use std::cell::Cell;
use std::cmp::PartialEq;
use self::LeafOrInternal::{Internal, Leaf, Root};

#[derive(PartialEq, Copy, Clone, Debug)]
enum Color {
    R,
    B
}


// TODO: nodes are local when they should be on the heap via Box

#[derive(Debug, Clone, Copy)]
enum LeafOrInternal<T> {
    Internal(T),
    Root,
    Leaf
}

impl PartialEq for LeafOrInternal<NonNull<Node>> {
    fn eq(&self, other: &LeafOrInternal<NonNull<Node>>) -> bool {
        true
    }
}


#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    color: Color,
    v: i32,
    parent: LeafOrInternal<NonNull<Node>>,
    left: LeafOrInternal<NonNull<Node>>,
    right: LeafOrInternal<NonNull<Node>>
}

// #[derive(Clone)]
// pub struct Tree {
//     root: NonNull<Node>
// }

// impl Tree {
//     fn new(data: i32) -> Node {
//         Node {
//             color: Color::B,
//             parent: Root,
//             left: Leaf,
//             right: Leaf,
//             v: data,
//         }
//     }

//     fn new_node(&mut self, data: i32) -> Node {
//         Node {
//             color: Color::B,
//             parent: Root,
//             left: Leaf,
//             right: Leaf,
//             v: data,
//         }
//     }


//     // pub fn get_root(&mut self) -> &mut Node {
//     //     &mut self.root
//     // }


// //     struct node *insert(struct node *root, struct node *n)
// //     {
// //         // insert new node into the current tree
// //         insert_recurse(root, n);

// //         // repair the tree in case any of the red-black properties have been violated
// //         insert_repair_tree(n);

// //         // find the new root to return
// //         root = n;
// //         while (parent(root) != NULL)
// //             root = parent(root);
// //         return root;
// // }




//     // pub fn verify_step(node: &Node) -> bool {
//     //     match node {
//     //         &Node::Node{ref color, ref left, ref right, ..} => {
//     //             match *color {
//     //                 Color::R => {
//     //                     let is_valid_left = Node::verify_step(&*left);
//     //                     let is_valid_right = Node::verify_step(&*right);
//     //                     (left.get_color() == Color::B
//     //                      && right.get_color() == Color::B
//     //                      && is_valid_right && is_valid_left )

//     //                 },
//     //                 Color::B => {
//     //                     let is_valid_left = Node::verify_step(&*left);
//     //                     let is_valid_right = Node::verify_step(&*right);
//     //                     (is_valid_right && is_valid_left)
//     //                 }
//     //             }
//     //         },
//     //         &Node::Leaf => {
//     //             true
//     //         }
//     //     }
//     // }


// }


impl LeafOrInternal<NonNull<Node>> {
    fn to_string(&self, mut d: usize) -> String {
        match *self {
            LeafOrInternal::Root => format!("{number:width$}  Root", number=d, width=d),
            LeafOrInternal::Leaf => {
                d += 1;
                format!("{number:width$}  Leaf", number=d, width=d)
            },
            LeafOrInternal::Internal(x) => unsafe {x.as_ref().to_string(d)},
        }
    }
}


impl Node {

    fn new(data: i32) -> Box<Node> {
        Box::new(Node {
            color: Color::B,
            parent: Root,
            left: Leaf,
            right: Leaf,
            v: data,
        })
    }

    fn new_node_color(data: i32, color: Color) -> Box<Node> {
        Box::new(Node {
            color: color,
            parent: Root,
            left: Leaf,
            right: Leaf,
            v: data,
        })
    }

    fn to_string(&self, mut d: usize) -> String {
        d += 1;
        format!("{number:width$} [{:?} {} [\n{} \n{}]]",
                self.color,
                self.v,
                self.left.to_string(d),
                self.right.to_string(d),
                number=d,
                width=d
        )
    }


    fn get_left(&self) -> Option<NonNull<Node>> {
        match self.left {
            LeafOrInternal::Leaf => None,
            LeafOrInternal::Root => None,
            LeafOrInternal::Internal(x) => Some(x)
        }
    }

    fn get_right(&self) -> Option<NonNull<Node>> {
        match self.right {
            LeafOrInternal::Leaf => None,
            LeafOrInternal::Root => None,
            LeafOrInternal::Internal(x) => Some(x)
        }
    }

    fn get_parent(&self) -> Option<NonNull<Node>> {
        match self.parent {
            LeafOrInternal::Leaf => None,
            LeafOrInternal::Root => None,
            LeafOrInternal::Internal(x) => Some(x)
        }
    }

    fn get_grandparent(&self) -> Option<NonNull<Node>> {
        self.get_parent().and_then(|x| unsafe {x.as_ref().get_parent()})
    }

    // //used in delete cases
    // unsafe fn get_sibling(&self) -> Option<&Node> {
    //     if let Some(parent) = self.parent.as_ref() {
    //         if *self == *(*parent).left {
    //             parent.as_ref().and_then(|x| x.right.as_ref())
    //         } else {
    //             parent.as_ref().and_then(|x| x.left.as_ref())
    //         }
    //     }
    // }

    unsafe fn get_immediate_family(&self) -> Option<(NonNull<Node>, NonNull<Node>)> {
        match self.get_parent() {
            Some(parent) => {
                match self.get_grandparent() {
                    Some(grandparent) => {
                        // TODO: left can be a LEAF, how is this handled
                        match grandparent.as_ref().get_left() {
                            Some(which_uncle) => {
                                if parent.as_ref() == which_uncle.as_ref() {
                                    let uncle = grandparent.as_ref().get_right();
                                    Some((grandparent, uncle.unwrap()))
                                } else {
                                    let uncle = grandparent.as_ref().get_left();
                                    Some((grandparent, uncle.unwrap()))
                                }
                            },
                            None => None

                        }

                    },
                    None => None
                }
            },
            None => None
        }
    }

    unsafe fn rotate_left(&mut self) {
        println!("------------ rot left");
        if let Internal(mut new_node_ptr) = self.right.clone() {
            let mut new_node_ptr_clone = new_node_ptr.clone();
            let mut new_node: &mut Node = new_node_ptr_clone.as_mut();
            new_node.parent = self.parent;
            self.parent = Internal(new_node_ptr);
            let old_left = mem::replace(&mut new_node.left, Internal(NonNull::new(self as *mut Node).unwrap()));
            self.right = old_left;
        }
    }

    unsafe fn rotate_right(&mut self) {
        println!("------------ rot right");
        if let Internal(mut new_node_ptr) = self.left.clone() {
            let mut new_node_ptr_clone = new_node_ptr.clone();
            let mut new_node: &mut Node = new_node_ptr_clone.as_mut();
            new_node.parent = self.parent;
            self.parent = Internal(new_node_ptr);
            let old_right = mem::replace(&mut new_node.right, Internal(NonNull::new(self as *mut Node).unwrap()));
            self.left = old_right;
        }
    }


    // fn get_uncle(&self) -> Option<Node> {
    //     match self.get_parent() {
    //         Some(p) => p.get_sibling(),
    //         None => None
    //     }
    // }

    // while let Some(parent) = loop_ref.get_parent() {
    //     next_root_ptr = parent;
    //     loop_ref = parent.as_ref();
    // }
    // next_root_ptr.as_mut()

    fn insert(mut root: &mut Node, v: i32) -> NonNull<Node> {
        println!("------------ insert {} ------------", v);
        let mut last_ptr;
        unsafe {
            if let Internal(mut added_node)= Node::insert_recurse(root, v){

                let mut ptr_added_node = added_node.as_ptr();
                // TODO: it's corrupted here already
                println!("------------ insert recursed ------------ {:?}", (*ptr_added_node).parent);
                println!("------------ insert recursed ------------ {:?}", (*ptr_added_node).v);
                println!("------------ insert recursed------------ {:?}", ptr_added_node);
                Node::insert_repair_tree(ptr_added_node);
                // TODO: somehow added_node is corrupted
                println!("------------ done repairs ------------ {:?}", added_node);
                println!("------------ done repairs ------------ {:?}", added_node.as_mut().v);
                let mut last = None;
                let mut traverse_node = added_node.as_ptr();
                if let Some(x) = (*traverse_node).get_parent() {
                    while let Some(x) = (*traverse_node).get_parent() {
                        println!("-------- loop {:?}", x.as_ref());
                        last = Some(x);
                        traverse_node = x.as_ptr();
                    }

                    println!("out of loop");
                    last_ptr = last.unwrap();
                } else {
                    last_ptr = added_node
                }
            } else {
                unreachable!();
            }
        }
        last_ptr
    }

    unsafe fn insert_recurse(mut root: &mut Node, in_v: i32) -> LeafOrInternal<NonNull<Node>> {
        println!("------------ insert recurse {} ------------", in_v);
        if in_v < root.v {
            match root.get_left() {
                Some(mut left) => {
                    Node::insert_recurse(left.as_mut(), in_v)
                },
                None => {
                    let mut node = Node::new_node_color(in_v, Color::R);
                    node.parent = Internal(NonNull::new(root as *mut Node).unwrap());

                    println!("------------ insert recurse parent {:?} ------------", node.parent);
                    root.left = Internal(NonNull::new(&mut *node as *mut Node).unwrap());
                    root.left
                }
            }
        } else {
            match root.get_right() {
                Some(mut right) => {
                    Node::insert_recurse(right.as_mut(), in_v)
                },
                None => {
                    let mut node = Node::new_node_color(in_v, Color::R);
                    node.parent = Internal(NonNull::new(root as *mut Node).unwrap());
                    println!("------------ insert recurse parent {:?} ------------", node.parent);
                    root.right = Internal(NonNull::new(&mut *node as *mut Node).unwrap());
                    root.right
                    // root.get_right()
                }
            }
        }
    }

    unsafe fn insert_repair_tree(mut node: *mut Node) // -> NonNull<Node>
    {
        // TODO: something seems to be falling under the wrong case
        // let node = node_ptr.as_ptr();
        println!("------------ insert repair ------------");
        // let mut out_node_ptr = node_ptr.clone();
        // println!("------------ insert repair ------------ {:?}", out_node_ptr);
        // let mut get_node = node_ptr.clone();
        // println!("------------ insert repair ------------ {:?}", get_node);
        // let mut node = node_ptr.as_mut();
        println!("------------ insert repair node ------------ {:?}", (*node).parent);
        println!("------------ insert repair node ------------ {:?}", node);
        println!("------------ insert repair node ------------ v {:?}", (*node).v);
        match (*node).get_parent() {
            None => {
                println!("------------ insert repair node recolor ------------ {:?}", (*node).v);
                (*node).color = Color::B;
                println!("------------ insert repair node recolor ------------ {:?}", (*node).v);
                // println!("------------ fix last node ------------ {:?}", node.color);
                // node_ptr
            },
            Some(mut parent_ptr) => {
                println!("------------ insert repair parent ------------");
                let out_parent_ptr = parent_ptr.clone();
                let parent = parent_ptr.as_mut();
                println!("------------ insert repair parent ------------ {:?}", parent);
                match parent.color {
                    Color::B => {
                        return;
                        // node_ptr
                    },
                    Color::R => {
                        match (*node).get_immediate_family() {
                            Some(( mut grandparent_ptr, mut uncle_ptr)) => {
                                println!("------------ insert repair immediate fam ------------");
                                let out_grandparent_ptr = grandparent_ptr.clone();
                                let grandparent = grandparent_ptr.as_mut();
                                let uncle = uncle_ptr.as_mut();
                                println!("------------ insert repair grand ------------ {:?}", grandparent);
                                println!("------------ insert repair uncle ------------ {:?}", uncle);
                                match uncle.color {
                                    Color::R => {
                                        parent.color = Color::B;
                                        uncle.color = Color::B;
                                        grandparent.color = Color::R;
                                        Node::insert_repair_tree(out_grandparent_ptr.as_ptr());
                                        // node_ptr
                                    },
                                    Color::B => {
                                        // return;
                                        Node::insert_repair_tree_case4(
                                            NonNull::new(node).unwrap(), out_parent_ptr, out_grandparent_ptr);
                                        // node_ptr
                                    }
                                }
                            },
                            None => {
                                // TODO: ???????????????????????????????????
                                return;
                                // unreachable!();
                                // node_ptr
                            }
                        }
                    }
                }
            }

        }
    }

    unsafe fn insert_repair_tree_case4(mut node_ptr: NonNull<Node>, mut parent_ptr: NonNull<Node>, mut grandparent_ptr: NonNull<Node>) {
        println!("------------ insert_repair_tree_case4 ------------");
        let mut next_ptr;
        let mut node = node_ptr.as_mut();
        let mut parent = parent_ptr.as_mut();
        let mut grandparent = grandparent_ptr.as_mut();


        if node == grandparent.get_left().unwrap().as_ref().get_right().unwrap().as_mut() {
            parent.rotate_left();
            next_ptr = node.get_left().unwrap();
            node = next_ptr.as_mut();
        } else if node == grandparent.get_right().unwrap().as_ref().get_left().unwrap().as_mut()  {
            parent.rotate_right();
            next_ptr = node.get_right().unwrap();
            node = next_ptr.as_mut();
        }

        if node == parent.get_left().unwrap().as_ref() {
            grandparent.rotate_right();
        } else {
            grandparent.rotate_left();
        }

        parent.color = Color::B;
        grandparent.color = Color::R;
    }



    fn verify_rb(&self) -> bool {
        assert!(self.color == Color::B);
        // Tree::verify_step(&root)
        true
    }


}




// Each node is either red or black.
//     The root is black. This rule is sometimes omitted. Since the root can always be changed from red to black, but not necessarily vice versa, this rule has little effect on analysis.
//     All leaves (NIL) are black.
//     If a node is red, then both its children are black.
//     Every path from a given node to any of its descendant NIL nodes contains the same number of black nodes. Some definitions: the number of black nodes from the root to a node is the node's black depth; the uniform number of black nodes in all paths from root to the leaves is called the black-height of the red–black tree.[17]



#[cfg(test)]
mod tests {
    use std::boxed::Box;
    use std::ptr;
    use std::ptr::NonNull;
    use super::{Node, Color};
    use super::LeafOrInternal;
    use super::LeafOrInternal::{Leaf, Internal, Root};

    fn new_node_color(data: i32, color: Color) -> Node {
        Node {
            color: color,
            parent: Root,
            left: Leaf,
            right: Leaf,
            v: data,
        }
    }

    fn verify_count(node_container: LeafOrInternal<NonNull<Node>>, mut blk_ctr: usize) -> usize {
        match node_container {
            LeafOrInternal::Root => blk_ctr,
            LeafOrInternal::Internal(n) => {
                unsafe {
                    let node = n.as_ref();
                    match node.color {
                        Color::R => {
                            let left_path_ctr = verify_count(node.left, blk_ctr);
                            let right_path_ctr = verify_count(node.right, blk_ctr);
                            assert_eq!(left_path_ctr, right_path_ctr);
                            blk_ctr
                        },
                        Color::B => {
                            blk_ctr += 1;
                            let left_path_ctr = verify_count(node.left, blk_ctr);
                            let right_path_ctr = verify_count(node.right, blk_ctr);
                            assert_eq!(left_path_ctr, right_path_ctr);
                            blk_ctr
                        }
                    }
                }
            },
            LeafOrInternal::Leaf => {
                blk_ctr
            }
        }
    }

    // #[test]
    // fn verify_root() {
    //     let mut node = Node::new(13);
    //     assert!(node.verify_rb());
    // }

    // #[test]
    // fn is_diagram_valid() {
    //     let mut a = Node::new(13);
    //     // let a = tree.new_node_color(13, Color::B);
    //     let mut b = new_node_color(8 , Color::R);
    //     let mut c = new_node_color(1 , Color::B);
    //     let mut d = new_node_color(11, Color::B);
    //     let mut e = new_node_color(6 , Color::R);
    //     let mut f = new_node_color(17, Color::R);
    //     let mut g = new_node_color(15, Color::B);
    //     let mut h = new_node_color(25, Color::B);
    //     let mut i = new_node_color(22, Color::R);
    //     let mut j = new_node_color(27, Color::R);

    //     let a_ptr: *mut Node = &mut a;
    //     b.parent = Internal(NonNull::new(a_ptr).unwrap());
    //     f.parent = Internal(NonNull::new(a_ptr).unwrap());

    //     let b_ptr: *mut Node = &mut b;
    //     c.parent = Internal(NonNull::new(b_ptr).unwrap());
    //     d.parent = Internal(NonNull::new(b_ptr).unwrap());

    //     let c_ptr: *mut Node = &mut c;

    //     e.parent = Internal(NonNull::new(c_ptr).unwrap());

    //     let h_ptr: *mut Node = &mut h;
    //     i.parent = Internal(NonNull::new(h_ptr).unwrap());
    //     j.parent = Internal(NonNull::new(h_ptr).unwrap());

    //     let f_ptr: *mut Node = &mut f;
    //     g.parent = Internal(NonNull::new(f_ptr).unwrap());
    //     h.parent = Internal(NonNull::new(f_ptr).unwrap());

    //     a.left = Internal(NonNull::new(&mut b as *mut Node).unwrap());
    //     a.right = Internal(NonNull::new(&mut f as *mut Node).unwrap());

    //     b.left = Internal(NonNull::new(&mut c as *mut Node).unwrap());
    //     b.right = Internal(NonNull::new(&mut d as *mut Node).unwrap());

    //     c.right = Internal(NonNull::new(&mut e as *mut Node).unwrap());

    //     f.left = Internal(NonNull::new(&mut g as *mut Node).unwrap());
    //     f.right = Internal(NonNull::new(&mut h as *mut Node).unwrap());

    //     h.left = Internal(NonNull::new(&mut i as *mut Node).unwrap());
    //     h.right =  Internal(NonNull::new(&mut j as *mut Node).unwrap());

    //         // let mut a = tree.root.as_mut().unwrap();
    //     println!("{}", a.to_string(0));
    //     assert!(a.verify_rb());
    //     verify_count(a.left, 0);
    //     verify_count(a.right, 0);
    // }


    // #[test]
    // fn test_rotations() {
    //     let mut a = Node::new(13);
    //     // let a = tree.new_node_color(13, Color::B);
    //     let mut b = new_node_color(8 , Color::R);
    //     let mut c = new_node_color(1 , Color::B);
    //     let mut d = new_node_color(11, Color::B);
    //     let mut e = new_node_color(6 , Color::R);
    //     let mut f = new_node_color(17, Color::R);
    //     let mut g = new_node_color(15, Color::B);
    //     let mut h = new_node_color(25, Color::B);
    //     let mut i = new_node_color(22, Color::R);
    //     let mut j = new_node_color(27, Color::R);

    //     let a_ptr: *mut Node = &mut a;
    //     b.parent = Internal(NonNull::new(a_ptr).unwrap());
    //     f.parent = Internal(NonNull::new(a_ptr).unwrap());

    //     let b_ptr: *mut Node = &mut b;
    //     c.parent = Internal(NonNull::new(b_ptr).unwrap());
    //     d.parent = Internal(NonNull::new(b_ptr).unwrap());

    //     let c_ptr: *mut Node = &mut c;

    //     e.parent = Internal(NonNull::new(c_ptr).unwrap());

    //     let h_ptr: *mut Node = &mut h;
    //     i.parent = Internal(NonNull::new(h_ptr).unwrap());
    //     j.parent = Internal(NonNull::new(h_ptr).unwrap());

    //     let f_ptr: *mut Node = &mut f;
    //     g.parent = Internal(NonNull::new(f_ptr).unwrap());
    //     h.parent = Internal(NonNull::new(f_ptr).unwrap());

    //     a.left = Internal(NonNull::new(&mut b as *mut Node).unwrap());
    //     a.right = Internal(NonNull::new(&mut f as *mut Node).unwrap());

    //     b.left = Internal(NonNull::new(&mut c as *mut Node).unwrap());
    //     b.right = Internal(NonNull::new(&mut d as *mut Node).unwrap());

    //     c.right = Internal(NonNull::new(&mut e as *mut Node).unwrap());

    //     f.left = Internal(NonNull::new(&mut g as *mut Node).unwrap());
    //     f.right = Internal(NonNull::new(&mut h as *mut Node).unwrap());

    //     h.left = Internal(NonNull::new(&mut i as *mut Node).unwrap());
    //     h.right =  Internal(NonNull::new(&mut j as *mut Node).unwrap());

    //     unsafe {
    //         b.rotate_right();
    //         c.rotate_right();
    //         c.rotate_left();
    //         f.rotate_left();
    //         f.rotate_right();

    //         println!("{}", a.to_string(0));
    //         assert!(a.verify_rb());
    //     }
    // }


    #[test]
    fn test_accessors() {
        let mut a = Node::new(13);
        // let a = tree.new_node_color(13, Color::B);
        let mut b = new_node_color(8 , Color::R);
        let mut c = new_node_color(1 , Color::B);
        let mut d = new_node_color(11, Color::B);
        let mut e = new_node_color(6 , Color::R);
        let mut f = new_node_color(17, Color::R);
        let mut g = new_node_color(15, Color::B);
        let mut h = new_node_color(25, Color::B);
        let mut i = new_node_color(22, Color::R);
        let mut j = new_node_color(27, Color::R);

        let a_ptr: *mut Node = Box::into_raw(a);
        b.parent = Internal(NonNull::new(a_ptr).unwrap());
        f.parent = Internal(NonNull::new(a_ptr).unwrap());

        let b_ptr: *mut Node = &mut b;
        c.parent = Internal(NonNull::new(b_ptr).unwrap());
        d.parent = Internal(NonNull::new(b_ptr).unwrap());

        let c_ptr: *mut Node = &mut c;

        e.parent = Internal(NonNull::new(c_ptr).unwrap());

        let h_ptr: *mut Node = &mut h;
        i.parent = Internal(NonNull::new(h_ptr).unwrap());
        j.parent = Internal(NonNull::new(h_ptr).unwrap());

        let f_ptr: *mut Node = &mut f;
        g.parent = Internal(NonNull::new(f_ptr).unwrap());
        h.parent = Internal(NonNull::new(f_ptr).unwrap());

        unsafe {
            (*a_ptr).left = Internal(NonNull::new(&mut b as *mut Node).unwrap());
            (*a_ptr).right = Internal(NonNull::new(&mut f as *mut Node).unwrap());
        }

        b.left = Internal(NonNull::new(&mut c as *mut Node).unwrap());
        b.right = Internal(NonNull::new(&mut d as *mut Node).unwrap());

        c.right = Internal(NonNull::new(&mut e as *mut Node).unwrap());

        f.left = Internal(NonNull::new(&mut g as *mut Node).unwrap());
        f.right = Internal(NonNull::new(&mut h as *mut Node).unwrap());

        h.left = Internal(NonNull::new(&mut i as *mut Node).unwrap());
        h.right =  Internal(NonNull::new(&mut j as *mut Node).unwrap());

        unsafe {
            let mut v = Node::insert(&mut *a_ptr, 38 );
            println!("inserted");
            verify_count(v.as_ref().left, 0);
            verify_count(v.as_ref().right, 0);
            println!("{}", v.as_ref().to_string(0));
        }
    }


    // #[test]
    // fn test_insert() {
    //     let mut a = Node::new(13);
    //     // let a = tree.new_node_color(13, Color::B);
    //     unsafe {
    //         let mut b = Node::insert(&mut a, 8 );
    //         // b = Node::insert(b.as_mut(), 1 );
    //         // a = Node::insert(a, 11).as_mut();
    //         // a = Node::insert(a, 6 ).as_mut();
    //         // a = Node::insert(a, 17).as_mut();
    //         // a = Node::insert(a, 15).as_mut();
    //         // a = Node::insert(a, 25).as_mut();
    //         // a = Node::insert(a, 22).as_mut();
    //         // a = Node::insert(a, 27).as_mut();

    //         println!("{}", b.as_ref().to_string(0));
    //         // assert!(a.verify_rb());
    //         // verify_count(a.left, 0);
    //         // verify_count(a.right, 0);
    //     }
    // }

    // #[test]
    // fn test_leaf_as_uncle() {
    //     let mut a = Node::new(13);
    //     // let a = tree.new_node_color(13, Color::B);
    //     let mut b = new_node_color(8 , Color::R);
    //     let mut c = new_node_color(1 , Color::B);
    //     let mut d = new_node_color(11, Color::B);
    //     let mut e = new_node_color(6 , Color::R);
    //     let mut f = new_node_color(17, Color::R);
    //     let mut g = new_node_color(15, Color::B);
    //     let mut h = new_node_color(25, Color::B);
    //     let mut i = new_node_color(22, Color::R);
    //     let mut j = new_node_color(27, Color::R);

    //     let a_ptr: *mut Node = Box::into_raw(a);
    //     b.parent = Internal(NonNull::new(a_ptr).unwrap());
    //     f.parent = Internal(NonNull::new(a_ptr).unwrap());

    //     let b_ptr: *mut Node = &mut b;
    //     c.parent = Internal(NonNull::new(b_ptr).unwrap());
    //     d.parent = Internal(NonNull::new(b_ptr).unwrap());

    //     let c_ptr: *mut Node = &mut c;

    //     e.parent = Internal(NonNull::new(c_ptr).unwrap());

    //     let h_ptr: *mut Node = &mut h;
    //     i.parent = Internal(NonNull::new(h_ptr).unwrap());
    //     j.parent = Internal(NonNull::new(h_ptr).unwrap());

    //     let f_ptr: *mut Node = &mut f;
    //     g.parent = Internal(NonNull::new(f_ptr).unwrap());
    //     h.parent = Internal(NonNull::new(f_ptr).unwrap());

    //     unsafe {
    //         (*a_ptr).left = Internal(NonNull::new(&mut b as *mut Node).unwrap());
    //         (*a_ptr).right = Internal(NonNull::new(&mut f as *mut Node).unwrap());
    //     }

    //     b.left = Internal(NonNull::new(&mut c as *mut Node).unwrap());
    //     b.right = Internal(NonNull::new(&mut d as *mut Node).unwrap());

    //     c.right = Internal(NonNull::new(&mut e as *mut Node).unwrap());

    //     f.left = Internal(NonNull::new(&mut g as *mut Node).unwrap());
    //     f.right = Internal(NonNull::new(&mut h as *mut Node).unwrap());

    //     h.left = Internal(NonNull::new(&mut i as *mut Node).unwrap());
    //     h.right =  Internal(NonNull::new(&mut j as *mut Node).unwrap());

    //     unsafe {
    //         let mut v = Node::insert(&mut *a_ptr, 3 );
    //         println!("inserted");
    //         verify_count(v.as_ref().left, 0);
    //         verify_count(v.as_ref().right, 0);
    //         println!("{}", v.as_ref().to_string(0));
    //     }
    // }
 
}
