#![feature(ptr_internals)]
#![feature(shared)]

// http://cglab.ca/~abeinges/blah/too-many-lists/book/
// reference http://cglab.ca/~abeinges/blah/rust-btree-case/
// https://www.reddit.com/r/rust/comments/3svacd/memory_corruption_of_a_raw_pointer/?st=je8oull8&sh=d9f97f78


// TODO: cleanup children references on drop
// TODO: get_left and right and such should return LeafOrInternal
// TODO: LeafOrInternal needs some methods similar to Option, such as and_then or unwrap

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
    fn to_debug(&self, mut d: usize) -> String {
        match *self {
            LeafOrInternal::Root => {
                format!("{number:width$}  Root", number=d, width=d)
            },
            LeafOrInternal::Leaf => {
                d += 1;
                format!("{number:width$}  Leaf", number=d, width=d)
            },
            LeafOrInternal::Internal(x) => {
                unsafe {x.as_ref().to_debug(d)}
            },
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

    fn to_debug(&self, mut d: usize) -> String {
        d += 1;
        format!("{number:width$} [{:?} {} [\n{} \n{}]]",
                self.color,
                self.v,
                self.left.to_debug(d),
                self.right.to_debug(d),
                number=d,
                width=d
        )
    }


    fn to_string(&self) -> String {
        format!("[{:?} {}]",
                self.color,
                self.v
                // self.left.to_debug(d),
                // self.right.to_debug(d),
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

    unsafe fn get_immediate_family(&self) -> Option<(*mut Node, *mut Node)> {
        match self.get_parent() {
            Some(parent) => {
                match self.get_grandparent() {
                    Some(grandparent) => {
                        // TODO: left can be a LEAF, how is this handled
                        match grandparent.as_ref().get_left() {
                            Some(left_uncle) => {
                                // println!("get immediate family compare: {} {}", parent.as_ref().v, left_uncle.as_ref().v);
                                if parent.as_ref().v == left_uncle.as_ref().v {
                                    match grandparent.as_ref().get_right() {
                                        Some(uncle) => {
                                            // println!("get immediate family compare uncle: {}", uncle.as_ref().v);
                                            Some((grandparent.as_ptr(), uncle.as_ptr()))
                                        },
                                        None => None
                                    }
                                } else {
                                    match grandparent.as_ref().get_left() {
                                        Some(uncle) => {
                                            // println!("get immediate family compare uncle: {}", uncle.as_ref().v);
                                            Some((grandparent.as_ptr(), uncle.as_ptr()))
                                        },
                                        None => None
                                    }
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
        // println!("------------ insert {} ------------", v);
        let mut last_ptr;
        unsafe {
            let added_node = Node::insert_recurse(root, v);
            // if let Internal(mut added_node)= ){
            // let mut ptr_added_node = added_node.as_ptr();
            // println!("insert pre repair {}", root.to_debug(0) );
            Node::insert_repair_tree(added_node);
            // println!("insert post repair {}", root.to_debug(0) );
            let mut last = None;
            let mut traverse_node = added_node;
            if let Some(x) = (*traverse_node).get_parent() {
                while let Some(x) = (*traverse_node).get_parent() {
                    // println!("-------- loop {:?}", x.as_ref());
                    last = Some(x);
                    traverse_node = x.as_ptr();
                }
                last_ptr = last.unwrap();
            } else {
                last_ptr = NonNull::new(added_node).unwrap()
            }
            // } else {
            //     unreachable!();
            // }
        }
        last_ptr
    }

    unsafe fn insert_recurse(mut root: *mut Node, in_v: i32) -> *mut Node {
        if in_v < (*root).v {
            match (*root).get_left() {
                Some(mut left) => {
                    Node::insert_recurse(left.as_ptr(), in_v)
                },
                None => {
                    let mut node = Node::new_node_color(in_v, Color::R);
                    node.parent = Internal(NonNull::new(root as *mut Node).unwrap());
                    (* root ).left = Internal(NonNull::new(&mut *node as *mut Node).unwrap());
                    Box::into_raw(node)
                }
            }
        } else {
            match (*root).get_right() {
                Some(mut right) => {
                    Node::insert_recurse(right.as_ptr(), in_v)
                },
                None => {
                    let mut node = Node::new_node_color(in_v, Color::R);
                    node.parent = Internal(NonNull::new(root as *mut Node).unwrap());
                    (* root ).right = Internal(NonNull::new(&mut *node as *mut Node).unwrap());
                    Box::into_raw(node)
                }
            }
        }
    }

    unsafe fn insert_repair_tree(mut node: *mut Node) {
        // println!("------------ insert repair ------------");
        match (*node).get_parent() {
            None => {
                // println!("------------ case 1 ------------");
                (*node).color = Color::B;
                return;
            },
            Some(mut parent_ptr) => {
                // let out_parent_ptr = parent_ptr.clone();
                let raw_parent_ptr = parent_ptr.as_ptr();
                // let parent = parent_ptr.as_mut();
                match (*raw_parent_ptr).color {
                    Color::B => {
                        // println!("------------ case 2 ------------");
                        return;
                    },
                    Color::R => {
                        // if no uncle - color is black

                        match (*node).get_immediate_family() {
                            Some(( mut raw_grandparent_ptr, mut raw_uncle_ptr)) => {

                                // let out_grandparent_ptr = grandparent_ptr.clone();
                                // let grandparent = grandparent_ptr.as_mut();
                                // let uncle = uncle_ptr.as_mut();

                                match (*raw_uncle_ptr).color {
                                    Color::R => {
                                        // println!("------------ case 3 ------------");
                                        (*raw_parent_ptr).color = Color::B;
                                        (*raw_uncle_ptr).color = Color::B;
                                        (*raw_grandparent_ptr).color = Color::R;

                                        // println!("------------ loop, new node ------------ {}", (*raw_grandparent_ptr).v);
                                        Node::insert_repair_tree(raw_grandparent_ptr);
                                        return;
                                        // node_ptr
                                    },
                                    Color::B => {
                                        // println!("------------ case 4 ------------");
                                        // return;
                                        Node::insert_repair_tree_case4(node, raw_parent_ptr, raw_grandparent_ptr);
                                        return;
                                    }
                                }
                            },
                            None => {
                                // Uncle was a leaf and therefore black
                                // println!("------------ no uncle case 4 ------------");
                                let raw_grandparent_ptr = (*raw_parent_ptr).get_parent().unwrap().as_ptr();
                                Node::insert_repair_tree_case4(node, raw_parent_ptr, raw_grandparent_ptr);
                                // return;
                                // TODO: ???????????????????????????????????
                                return;
                            }
                        }
                    }
                }
            }

        }
    }

    unsafe fn insert_repair_tree_case4(mut node: *mut Node, mut parent: *mut Node, mut grandparent: *mut Node) {
        // println!("------------ insert_repair_tree_case4 ------------");
        let mut next_ptr;
        // let mut node = node_ptr.as_mut();
        // let mut parent = parent_ptr.as_mut();
        // let mut grandparent = grandparent_ptr.as_mut();


        if Some((node)) == (*grandparent).get_left().and_then(|x| x.as_ref().get_right().map(|v| v.as_ptr())) {
            (*parent).rotate_left();
            next_ptr = (*node).get_left().unwrap();
            node = next_ptr.as_ptr();
        } else if Some((node)) == (*grandparent).get_right().and_then(|x| x.as_ref().get_left().map(|v| v.as_ptr()))  {
            (*parent).rotate_right();
            next_ptr = (*node).get_right().unwrap();
            node = next_ptr.as_ptr();
        }

        if Some((node)) == (*parent).get_left().map(|x| x.as_ptr()) {
            (*grandparent).rotate_right();
        } else {
            (* grandparent ).rotate_left();
        }

        (* parent ).color = Color::B;
        (* grandparent ).color = Color::R;
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

    fn new_node_color(data: i32, color: Color) -> Box<Node> {
        Box::new(Node {
            color: color,
            parent: Root,
            left: Leaf,
            right: Leaf,
            v: data,
        })
    }

    fn verify_count(node_container: LeafOrInternal<NonNull<Node>>, mut blk_ctr: usize) -> usize {
        match node_container {
            LeafOrInternal::Root => 0,
            LeafOrInternal::Internal(n) => {
                unsafe {
                    let node = n.as_ref();
                    match node.color {
                        Color::R => {
                            let left_path_ctr = verify_count(node.left, blk_ctr);
                            let right_path_ctr = verify_count(node.right, blk_ctr);
                            assert_eq!(left_path_ctr, right_path_ctr);
                            left_path_ctr
                        },
                        Color::B => {
                            blk_ctr += 1;
                            let left_path_ctr = verify_count(node.left, blk_ctr);
                            let right_path_ctr = verify_count(node.right, blk_ctr);
                            assert_eq!(left_path_ctr, right_path_ctr);
                            left_path_ctr + 1
                        }
                    }
                }
            },
            LeafOrInternal::Leaf => {
                0
            }
        }
    }

    // #[test]
    // fn verify_root() {
    //     let mut node = Node::new(13);
    //     assert!(node.verify_rb());
    // }


    #[test]
    fn test_rotate_left() {
        let a = Node::new(13);
        let f = new_node_color(17, Color::R);
        let h = new_node_color(25, Color::B);
        let i = new_node_color(22, Color::R);

        let a_ptr: *mut Node = Box::into_raw(a);
        let f_ptr: *mut Node = Box::into_raw(f);
        let h_ptr: *mut Node = Box::into_raw(h);
        let i_ptr: *mut Node = Box::into_raw(i);

        unsafe {
            (*f_ptr).parent = Internal(NonNull::new(a_ptr).unwrap());
            (*i_ptr).parent = Internal(NonNull::new(h_ptr).unwrap());
            (*h_ptr).parent = Internal(NonNull::new(f_ptr).unwrap());
            (*a_ptr).right = Internal(NonNull::new(f_ptr).unwrap());
            (*f_ptr).right = Internal(NonNull::new(h_ptr).unwrap());
            (*h_ptr).left = Internal(NonNull::new(i_ptr).unwrap());

        }

        unsafe {
            //                                -- f, h, i, a
            // struct node *nnew = n->right;  -- f.right (h)
            // n->right = nnew->left;         -- f.right (h) <- h.left (i)
            // nnew->left = n;                -- h.left (i) <- f
            // nnew->parent = n->parent;      -- h.parent (f) <- f.parent (a)
            // n->parent = nnew;              -- f.parent (a) <- h

            (*f_ptr).rotate_left();
            assert_eq!((*f_ptr).get_right().unwrap().as_ref().v, 22);
            assert_eq!((*h_ptr).get_left().unwrap().as_ref().v, 17);
            assert_eq!((*h_ptr).get_parent().unwrap().as_ref().v, 13);
            assert_eq!((*f_ptr).get_parent().unwrap().as_ref().v, 25);
        }
    }

    #[test]
    fn test_rotate_right() {
        let a = Node::new(13);
        let f = new_node_color(17, Color::R);
        let g = new_node_color(15, Color::B);
        let h = new_node_color(25, Color::B);

        let a_ptr: *mut Node = Box::into_raw(a);
        let f_ptr: *mut Node = Box::into_raw(f);
        let g_ptr: *mut Node = Box::into_raw(g);
        let h_ptr: *mut Node = Box::into_raw(h);


        unsafe {
            (*f_ptr).parent = Internal(NonNull::new(a_ptr).unwrap());
            (*g_ptr).parent = Internal(NonNull::new(f_ptr).unwrap());
            (*a_ptr).right = Internal(NonNull::new(f_ptr).unwrap());
            (*f_ptr).left = Internal(NonNull::new(g_ptr).unwrap());
            (*f_ptr).right = Internal(NonNull::new(h_ptr).unwrap());
        }

        unsafe {
            //                                -- f, g, LEAF, a
            // struct node *nnew = n->left;   -- f.left (g)
            // n->left = nnew->right;         -- f.right (g) <- g.right (LEAF)
            // nnew->right = n;               -- g.right (LEAF) <- f
            // nnew->parent = n->parent;      -- g.parent (f) <- f.parent (a)
            // n->parent = nnew;              -- f.parent (a) <- g

            (*f_ptr).rotate_right();
            assert_eq!((*f_ptr).get_left(), None);
            assert_eq!((*g_ptr).get_right().unwrap().as_ref().v, 17);
            assert_eq!((*g_ptr).get_parent().unwrap().as_ref().v, 13);
            assert_eq!((*f_ptr).get_parent().unwrap().as_ref().v, 15);
        }
    }


    #[test]
    fn is_diagram_valid() {
        let a = Node::new(13);
        // let a = tree.new_node_color(13, Color::B);
        let b = new_node_color(8 , Color::R);
        let c = new_node_color(1 , Color::B);
        let d = new_node_color(11, Color::B);
        let e = new_node_color(6 , Color::R);
        let f = new_node_color(17, Color::R);
        let g = new_node_color(15, Color::B);
        let h = new_node_color(25, Color::B);
        let i = new_node_color(22, Color::R);
        let j = new_node_color(27, Color::R);

        let a_ptr: *mut Node = Box::into_raw(a);
        let b_ptr: *mut Node = Box::into_raw(b);
        let c_ptr: *mut Node = Box::into_raw(c);
        let d_ptr: *mut Node = Box::into_raw(d);
        let e_ptr: *mut Node = Box::into_raw(e);
        let f_ptr: *mut Node = Box::into_raw(f);
        let g_ptr: *mut Node = Box::into_raw(g);
        let h_ptr: *mut Node = Box::into_raw(h);
        let i_ptr: *mut Node = Box::into_raw(i);
        let j_ptr: *mut Node = Box::into_raw(j);

        unsafe {
            (*b_ptr).parent = Internal(NonNull::new(a_ptr).unwrap());
            (*f_ptr).parent = Internal(NonNull::new(a_ptr).unwrap());

            (*c_ptr).parent = Internal(NonNull::new(b_ptr).unwrap());
            (*d_ptr).parent = Internal(NonNull::new(b_ptr).unwrap());


            (*e_ptr).parent = Internal(NonNull::new(c_ptr).unwrap());

            (*i_ptr).parent = Internal(NonNull::new(h_ptr).unwrap());
            (*j_ptr).parent = Internal(NonNull::new(h_ptr).unwrap());

            (*g_ptr).parent = Internal(NonNull::new(f_ptr).unwrap());
            (*h_ptr).parent = Internal(NonNull::new(f_ptr).unwrap());

            (*a_ptr).left = Internal(NonNull::new(b_ptr).unwrap());
            (*a_ptr).right = Internal(NonNull::new(f_ptr).unwrap());

            (*b_ptr).left = Internal(NonNull::new(c_ptr).unwrap());
            (*b_ptr).right = Internal(NonNull::new(d_ptr).unwrap());

            (*c_ptr).right = Internal(NonNull::new(e_ptr).unwrap());

            (*f_ptr).left = Internal(NonNull::new(g_ptr).unwrap());
            (*f_ptr).right = Internal(NonNull::new(h_ptr).unwrap());

            (*h_ptr).left = Internal(NonNull::new(i_ptr).unwrap());
            (*h_ptr).right =  Internal(NonNull::new(j_ptr).unwrap());

        }

            // let mut a = tree.root.as_mut().unwrap();
        unsafe {
            // println!("{}", (*a_ptr).to_debug(0));
            assert!((*a_ptr).verify_rb());
            verify_count((*a_ptr).left, 0);
            verify_count((*a_ptr).right, 0);
        }
    }


    #[test]
    fn test_multiple_rotations_ptrs_consistent() {
        let a = Node::new(13);
        // let a = tree.new_node_color(13, Color::B);
        let b = new_node_color(8 , Color::R);
        let c = new_node_color(1 , Color::B);
        let d = new_node_color(11, Color::B);
        let e = new_node_color(6 , Color::R);
        let f = new_node_color(17, Color::R);
        let g = new_node_color(15, Color::B);
        let h = new_node_color(25, Color::B);
        let i = new_node_color(22, Color::R);
        let j = new_node_color(27, Color::R);

        let a_ptr: *mut Node = Box::into_raw(a);
        let b_ptr: *mut Node = Box::into_raw(b);
        let c_ptr: *mut Node = Box::into_raw(c);
        let d_ptr: *mut Node = Box::into_raw(d);
        let e_ptr: *mut Node = Box::into_raw(e);
        let f_ptr: *mut Node = Box::into_raw(f);
        let g_ptr: *mut Node = Box::into_raw(g);
        let h_ptr: *mut Node = Box::into_raw(h);
        let i_ptr: *mut Node = Box::into_raw(i);
        let j_ptr: *mut Node = Box::into_raw(j);

        unsafe {
            (*b_ptr).parent = Internal(NonNull::new(a_ptr).unwrap());
            (*f_ptr).parent = Internal(NonNull::new(a_ptr).unwrap());

            (*c_ptr).parent = Internal(NonNull::new(b_ptr).unwrap());
            (*d_ptr).parent = Internal(NonNull::new(b_ptr).unwrap());


            (*e_ptr).parent = Internal(NonNull::new(c_ptr).unwrap());

            (*i_ptr).parent = Internal(NonNull::new(h_ptr).unwrap());
            (*j_ptr).parent = Internal(NonNull::new(h_ptr).unwrap());

            (*g_ptr).parent = Internal(NonNull::new(f_ptr).unwrap());
            (*h_ptr).parent = Internal(NonNull::new(f_ptr).unwrap());

            (*a_ptr).left = Internal(NonNull::new(b_ptr).unwrap());
            (*a_ptr).right = Internal(NonNull::new(f_ptr).unwrap());

            (*b_ptr).left = Internal(NonNull::new(c_ptr).unwrap());
            (*b_ptr).right = Internal(NonNull::new(d_ptr).unwrap());

            (*c_ptr).right = Internal(NonNull::new(e_ptr).unwrap());

            (*f_ptr).left = Internal(NonNull::new(g_ptr).unwrap());
            (*f_ptr).right = Internal(NonNull::new(h_ptr).unwrap());

            (*h_ptr).left = Internal(NonNull::new(i_ptr).unwrap());
            (*h_ptr).right =  Internal(NonNull::new(j_ptr).unwrap());

        }

        unsafe {
            (* b_ptr ).rotate_right();
            (* c_ptr ).rotate_right();
            (* c_ptr ).rotate_left();
            (* f_ptr ).rotate_left();
            (* f_ptr ).rotate_right();
        }
    }


    #[test]
    fn test_single_insert() {
        let a = Node::new(13);
        // let a = tree.new_node_color(13, Color::B);
        let b = new_node_color(8 , Color::R);
        let c = new_node_color(1 , Color::B);
        let d = new_node_color(11, Color::B);
        let e = new_node_color(6 , Color::R);
        let f = new_node_color(17, Color::R);
        let g = new_node_color(15, Color::B);
        let h = new_node_color(25, Color::B);
        let i = new_node_color(22, Color::R);
        let j = new_node_color(27, Color::R);

        let a_ptr: *mut Node = Box::into_raw(a);
        let b_ptr: *mut Node = Box::into_raw(b);
        let c_ptr: *mut Node = Box::into_raw(c);
        let d_ptr: *mut Node = Box::into_raw(d);
        let e_ptr: *mut Node = Box::into_raw(e);
        let f_ptr: *mut Node = Box::into_raw(f);
        let g_ptr: *mut Node = Box::into_raw(g);
        let h_ptr: *mut Node = Box::into_raw(h);
        let i_ptr: *mut Node = Box::into_raw(i);
        let j_ptr: *mut Node = Box::into_raw(j);

        unsafe {
            (*b_ptr).parent = Internal(NonNull::new(a_ptr).unwrap());
            (*f_ptr).parent = Internal(NonNull::new(a_ptr).unwrap());

            (*c_ptr).parent = Internal(NonNull::new(b_ptr).unwrap());
            (*d_ptr).parent = Internal(NonNull::new(b_ptr).unwrap());


            (*e_ptr).parent = Internal(NonNull::new(c_ptr).unwrap());

            (*i_ptr).parent = Internal(NonNull::new(h_ptr).unwrap());
            (*j_ptr).parent = Internal(NonNull::new(h_ptr).unwrap());

            (*g_ptr).parent = Internal(NonNull::new(f_ptr).unwrap());
            (*h_ptr).parent = Internal(NonNull::new(f_ptr).unwrap());

            (*a_ptr).left = Internal(NonNull::new(b_ptr).unwrap());
            (*a_ptr).right = Internal(NonNull::new(f_ptr).unwrap());

            (*b_ptr).left = Internal(NonNull::new(c_ptr).unwrap());
            (*b_ptr).right = Internal(NonNull::new(d_ptr).unwrap());

            (*c_ptr).right = Internal(NonNull::new(e_ptr).unwrap());

            (*f_ptr).left = Internal(NonNull::new(g_ptr).unwrap());
            (*f_ptr).right = Internal(NonNull::new(h_ptr).unwrap());

            (*h_ptr).left = Internal(NonNull::new(i_ptr).unwrap());
            (*h_ptr).right =  Internal(NonNull::new(j_ptr).unwrap());

        }

        unsafe {
            let mut v = Node::insert(&mut *a_ptr, 50 );
            // println!("inserted");

            verify_count(v.as_ref().left, 1);
            verify_count(v.as_ref().right, 1);

            // println!("{:?}", v);
            // verify_count(v.as_ref().left, 1);
            // verify_count(v.as_ref().right, 1);
            // println!("{}", v.as_ref().to_string());
            // println!("debug {}", v.as_ref().to_debug(0));
        }
    }


    #[test]
    fn test_converge_insert() {
        let mut x = Node::new(13);

        let mut a = Node::new(13);
        // let a = tree.new_node_color(13, Color::B);
        let b = new_node_color(8 , Color::R);
        let c = new_node_color(1 , Color::B);
        let d = new_node_color(11, Color::B);
        let e = new_node_color(6 , Color::R);
        let f = new_node_color(17, Color::R);
        let g = new_node_color(15, Color::B);
        let h = new_node_color(25, Color::B);
        let i = new_node_color(22, Color::R);
        let j = new_node_color(27, Color::R);


        let x_ptr: *mut Node = Box::into_raw(x);

        let a_ptr: *mut Node = Box::into_raw(a);
        let b_ptr: *mut Node = Box::into_raw(b);
        let c_ptr: *mut Node = Box::into_raw(c);
        let d_ptr: *mut Node = Box::into_raw(d);
        let e_ptr: *mut Node = Box::into_raw(e);
        let f_ptr: *mut Node = Box::into_raw(f);
        let g_ptr: *mut Node = Box::into_raw(g);
        let h_ptr: *mut Node = Box::into_raw(h);
        let i_ptr: *mut Node = Box::into_raw(i);
        let j_ptr: *mut Node = Box::into_raw(j);

        unsafe {
            (*b_ptr).parent = Internal(NonNull::new(a_ptr).unwrap());
            (*f_ptr).parent = Internal(NonNull::new(a_ptr).unwrap());

            (*c_ptr).parent = Internal(NonNull::new(b_ptr).unwrap());
            (*d_ptr).parent = Internal(NonNull::new(b_ptr).unwrap());


            (*e_ptr).parent = Internal(NonNull::new(c_ptr).unwrap());

            (*i_ptr).parent = Internal(NonNull::new(h_ptr).unwrap());
            (*j_ptr).parent = Internal(NonNull::new(h_ptr).unwrap());

            (*g_ptr).parent = Internal(NonNull::new(f_ptr).unwrap());
            (*h_ptr).parent = Internal(NonNull::new(f_ptr).unwrap());

            (*a_ptr).left = Internal(NonNull::new(b_ptr).unwrap());
            (*a_ptr).right = Internal(NonNull::new(f_ptr).unwrap());

            (*b_ptr).left = Internal(NonNull::new(c_ptr).unwrap());
            (*b_ptr).right = Internal(NonNull::new(d_ptr).unwrap());

            (*c_ptr).right = Internal(NonNull::new(e_ptr).unwrap());

            (*f_ptr).left = Internal(NonNull::new(g_ptr).unwrap());
            (*f_ptr).right = Internal(NonNull::new(h_ptr).unwrap());

            (*h_ptr).left = Internal(NonNull::new(i_ptr).unwrap());
            (*h_ptr).right =  Internal(NonNull::new(j_ptr).unwrap());

        }

        unsafe {
            let mut y_ptr = Node::insert(&mut *x_ptr, 8  ).as_ptr();

            // println!("debug {}", (*y_ptr).to_debug(0));
            y_ptr = Node::insert(&mut *y_ptr, 1  ).as_ptr();
            y_ptr = Node::insert(&mut *y_ptr, 11 ).as_ptr();
            y_ptr = Node::insert(&mut *y_ptr, 6  ).as_ptr();
            y_ptr = Node::insert(&mut *y_ptr, 17 ).as_ptr();
            y_ptr = Node::insert(&mut *y_ptr, 15 ).as_ptr();
            y_ptr = Node::insert(&mut *y_ptr, 25 ).as_ptr();
            y_ptr = Node::insert(&mut *y_ptr, 22 ).as_ptr();
            y_ptr = Node::insert(&mut *y_ptr, 27 ).as_ptr();

            // verify_count(x.as_ref().left, 1);
            // verify_count(x.as_ref().right, 1);

            assert_eq!((*y_ptr).to_debug(0), (*a_ptr).to_debug(0));
            // println!("debug {}", );
        }
    }
}
