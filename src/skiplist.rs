use std::rc::Rc;
use std::cell::RefCell;
use std::fmt::Debug;
use rand::prelude::*;

#[derive(Debug)]
pub struct KVPair<K, V> {
    key: K,
    value: V,
}

type Link<K, V> = Option<Rc<RefCell<Node<K,V>>>>;

#[derive(Debug)]
pub struct Node<K, V> {
    elem: KVPair<K, V>,
    nexts: Vec<Link<K, V>>
}


impl<K: std::cmp::PartialOrd + Copy, V: Copy> KVPair<K, V> {
    pub fn new(key: K, value: V) -> Self {
        KVPair { key, value } 
    } 
    pub fn greater(&self, other: &KVPair<K, V>) -> bool { self.key > other.key }
}

impl<K: std::cmp::PartialOrd + Copy, V: Copy> Node<K, V> {
    pub fn new(elem: KVPair<K, V>, level: usize) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Node { elem, nexts: vec![None; level] }))
    }
}

pub struct Skiplist<K, V> {
    heads: Vec<Link<K, V>>,
    top_level: Option<usize>,
    rng: ThreadRng,
}


impl<K: std::cmp::PartialOrd + Copy + Debug, V: Copy + Debug> Skiplist<K, V> {

    pub fn new() -> Self {
        Skiplist { heads: vec![None; 21], top_level: None, rng: rand::thread_rng()  }
    }

    fn _generate_level(&mut self) -> usize {
        let mut level = 0;
        let mut r: f64 = self.rng.gen();
        while r > 0.5 && level < 20 {
            level += 1;
            r = self.rng.gen();
        }
        level
    }


    pub fn insert(&mut self, key: &K, value: &V) -> Result<(), ()> {
        let elem = KVPair::new(*key, *value);
        println!("_insert {:?} {:?}", key, value);
        self._insert_internal(elem)
    }

    fn _insert_internal(&mut self, elem: KVPair<K, V>) -> Result<(), ()> {
        let mut frame = vec![None; 21];
        let mut node = self.find_no_greater_than(&elem.key, Some(&mut frame));
        let level = self._generate_level();

        println!("_insert {:?} level {:?}", &elem, level);
        println!("frame {:?}", frame);

        
        match node.take() {
            Some(node) => {
                if node.borrow().elem.key == elem.key {
                    return Err(());
                } else {
                    let new_node = Node::new(elem, level + 1);
                    let mut new_node_mut = new_node.borrow_mut();
                    new_node_mut.nexts[0] = node.borrow().nexts[0].clone();
                    node.borrow_mut().nexts[0] = Some(new_node.clone());
                }
            },
            None => {
                let new_node = Node::new(elem, level + 1);
                let mut new_node_mut = new_node.borrow_mut();
                let mut i = 0;
                
                while i <= level {
                    //println!("insert {:?} in heads level {}", new_node, i);
                    new_node_mut.nexts[i] = self.heads[i].clone();
                    self.heads[i] = Some(new_node.clone());
                    i += 1;
                }
            }
        }
        if self.top_level.is_none() || level > self.top_level.unwrap() {
            self.top_level = Some(level);
        }
        Ok(())
    }

    pub fn get(&self, key: &K) -> Option<V> {
        let node = self.find_no_greater_than(key, None);

        if let Some(node) = node {
            if node.borrow().elem.key == *key {
                Some(node.borrow().elem.value)
            } else {
                None
            }
        } else {
            None
        }
    }


    fn find_no_greater_than(&self, key: &K, mut frame: Option<&mut Vec<Link<K, V>>>) -> Link<K, V> {

        let level = self.top_level;

        if level == None {
            return None;
        }

        let mut level = level.unwrap();


        //let mut cur_node = self.heads[0].clone().unwrap();
        let mut cur_node: Link<K, V> = None;
        let mut next_node: Link<K, V>;

        println!("find_no_greater_than {:?}", key);
        println!("head node {:?}", self.heads);



        loop {
            next_node = match cur_node {
                Some(ref node) => node.borrow().nexts[level].clone(),
                None => self.heads[level].clone(),
            };
            match next_node.take() {
                Some(node) => {
                    println!("search node {:?}", node);
                    if node.borrow().elem.key <= *key {
                        cur_node = Some(node);
                        continue;
                    } else {
                        if let Some(ref mut f) = frame {
                            f[level] = cur_node.clone();
                        }
                    }
                }
                None => {
                    if let Some(ref mut f) = frame {
                        f[level] = cur_node.clone();
                    }
                }
            }
            if level > 0 {
                level -= 1;
            } else {
                break;
            }

        };
        println!("find node {:?}", cur_node);
        match cur_node {
            Some(node) => Some(node.clone()),
            None => None
        }
    }

}


#[cfg(test)]
mod test {
    use super::Skiplist;

    #[test]
    fn basic() {
        let mut list = Skiplist::<i64, i64>::new();
        
        println!("start");

        assert_eq!(list.get(&1), None);

        list.insert(&1, &1).ok();
        list.insert(&2, &2).ok();
        list.insert(&3, &3).ok();
        list.insert(&5, &5).ok();

        assert_eq!(list.get(&1), Some(1));
        assert_eq!(list.get(&2), Some(2));
        assert_eq!(list.get(&3), Some(3));
        assert_eq!(list.get(&5), Some(5));
        assert_eq!(list.insert(&5, &1), Err(()));
        assert_eq!(list.get(&4), None);
        assert_eq!(list.get(&6), None);
    }
}

