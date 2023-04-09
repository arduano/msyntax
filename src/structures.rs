// use std::{cell::Cell, collections::HashMap, rc::Rc};

// use crate::{Group, Match, Rule, Term, Token};

// #[derive(Debug, Eq, PartialEq, Hash, Clone)]
// pub(super) enum StructureValue<Index> {
//     Structure(Index),
//     Token(Token),
//     Group(Group, Index),
// }

// #[derive(Debug, Eq, PartialEq, Hash, Clone)]
// pub(super) enum StructureIndexed<Index> {
//     Product(ProductStructure<Index>),
//     Sum(SumStructure<Index>),
// }

// pub(super) type Structure = StructureIndexed<usize>;

// #[derive(Debug, Eq, PartialEq, Hash, Clone)]
// pub(super) struct ProductStructure<Index> {
//     pub values: Vec<StructureValue<Index>>,
// }

// #[derive(Debug, Eq, PartialEq, Hash, Clone)]
// pub(super) struct SumStructure<Index> {
//     pub variants: Vec<Index>,
// }

// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
// enum PotentialIndex {
//     Temp(usize),
//     Real(usize),
// }

// type SharedIndex = Rc<Cell<PotentialIndex>>;

// struct StructureBuilder {
//     structures: Vec<StructureIndexed<SharedIndex>>,
//     structure_for_rule: HashMap<Rule, SharedIndex>,
//     // next_tokens_for_rule: HashMap<Rule, Vec<Token>>,
//     temp_id_counter: usize,
// }

// /// Returns true if the slices contain the same elements, regardless of order.
// /// Assumes that the slices do not contain duplicates.
// fn are_slices_equal_unique_unordered<T: PartialEq>(a: &[T], b: &[T]) -> bool {
//     if a.len() != b.len() {
//         return false;
//     }

//     for a in a {
//         if !b.contains(a) {
//             return false;
//         }
//     }

//     true
// }

// /// Removes duplicate elements from a vector.
// fn deduplicate_vec<T: PartialEq>(vec: &mut Vec<T>) {
//     for i in 0..vec.len() {
//         for j in (i + 1)..vec.len() {
//             if vec[i] == vec[j] {
//                 vec.swap_remove(j);
//             }
//         }
//     }
// }

// fn new_real_index(index: usize) -> SharedIndex {
//     let potential = PotentialIndex::Real(index);
//     Rc::new(Cell::new(potential))
// }

// pub(super) fn build_structures(matches: &[Match]) -> Vec<Structure> {
//     let mut builder = StructureBuilder {
//         structures: Vec::new(),
//         structure_for_rule: HashMap::new(),
//         temp_id_counter: 0,
//     };

//     for m in matches {
//         builder.get_or_create_structure_for_rule(m.rule, matches);
//     }

//     for m in matches {
//         for i in 1..=m.terms.len() {
//             let trimmed = &m.terms[..i];
//             builder.get_or_create_structure_for_terms(trimmed, matches);
//         }
//     }

//     // Map strucures to their final index
//     builder
//         .structures
//         .into_iter()
//         .map(|s| s.get_structure())
//         .collect()
// }

// impl StructureBuilder {
//     fn next_shared_id(&mut self) -> SharedIndex {
//         let id = self.temp_id_counter;
//         self.temp_id_counter += 1;
//         let potential = PotentialIndex::Temp(id);
//         Rc::new(Cell::new(potential))
//     }

//     fn get_or_create_product_structure(
//         &mut self,
//         values: Vec<StructureValue<SharedIndex>>,
//     ) -> SharedIndex {
//         if values.len() == 1 {
//             if let StructureValue::Structure(index) = &values[0] {
//                 return index.clone();
//             }
//         }

//         for (i, structure) in self.structures.iter().enumerate() {
//             if let StructureIndexed::Product(product) = structure {
//                 if product.values == values {
//                     return new_real_index(i);
//                 }
//             }
//         }

//         let shared_index = new_real_index(self.structures.len());
//         self.structures
//             .push(StructureIndexed::Product(ProductStructure { values }));
//         shared_index
//     }

//     fn get_or_create_sum_structure(&mut self, values: Vec<SharedIndex>) -> SharedIndex {
//         for (i, structure) in self.structures.iter().enumerate() {
//             if let StructureIndexed::Sum(sum) = structure {
//                 if are_slices_equal_unique_unordered(&sum.variants, &values) {
//                     return new_real_index(i);
//                 }
//             }
//         }

//         let shared_index = new_real_index(self.structures.len());
//         self.structures
//             .push(StructureIndexed::Sum(SumStructure { variants: values }));
//         shared_index
//     }

//     fn get_or_create_structure_for_rule(
//         &mut self,
//         rule: Rule,
//         all_matches: &[Match],
//     ) -> SharedIndex {
//         if let Some(index) = self.structure_for_rule.get(&rule) {
//             return index.clone();
//         }

//         // Insert with a placeholder
//         let next_shared_id = self.next_shared_id();
//         self.structure_for_rule.insert(rule, next_shared_id.clone());

//         // Get all matches with this rule
//         let matches = all_matches
//             .iter()
//             .filter(|m| m.rule == rule)
//             .collect::<Vec<_>>();

//         let index = if matches.len() == 1 {
//             // If there is only one match, we can create a product structure
//             let values = matches[0]
//                 .terms
//                 .iter()
//                 .map(|t| self.get_or_create_structure_for_term(*t, all_matches))
//                 .collect();

//             self.get_or_create_product_structure(values)
//         } else {
//             // Otherwise, we need to create a sum structure
//             let mut variants = matches
//                 .iter()
//                 .map(|m| self.get_or_create_structure_for_terms(&m.terms, all_matches))
//                 .collect();

//             deduplicate_vec(&mut variants);

//             if variants.len() == 1 {
//                 // If there is only one variant, we can just return that
//                 variants.pop().unwrap()
//             } else {
//                 self.get_or_create_sum_structure(variants)
//             }
//         };

//         // Replace the placeholder
//         let index_inner = index.get();
//         next_shared_id.set(index_inner);

//         index
//     }

//     fn get_or_create_structure_for_term(
//         &mut self,
//         term: Term,
//         all_matches: &[Match],
//     ) -> StructureValue<SharedIndex> {
//         let value = match term {
//             Term::Rule(rule) => {
//                 let rule_index = self.get_or_create_structure_for_rule(rule, all_matches);
//                 let index = self
//                     .get_or_create_product_structure(vec![StructureValue::Structure(rule_index)]);
//                 StructureValue::Structure(index)
//             }
//             Term::Token(token) => StructureValue::Token(token),
//             Term::Group(group, rule) => {
//                 let rule_index = self.get_or_create_structure_for_rule(rule, all_matches);
//                 let index = self
//                     .get_or_create_product_structure(vec![StructureValue::Structure(rule_index)]);
//                 StructureValue::Group(group, index)
//             }
//         };

//         value
//     }

//     fn get_or_create_structure_for_terms(
//         &mut self,
//         terms: &[Term],
//         all_matches: &[Match],
//     ) -> SharedIndex {
//         let values = terms
//             .iter()
//             .map(|t| self.get_or_create_structure_for_term(*t, all_matches))
//             .collect();

//         self.get_or_create_product_structure(values)
//     }
// }

// fn force_unwrap_shared_index(index: &SharedIndex) -> usize {
//     match index.get() {
//         PotentialIndex::Temp(id) => unreachable!("Temp id {id} is still present"),
//         PotentialIndex::Real(id) => id,
//     }
// }

// impl StructureIndexed<SharedIndex> {
//     fn get_structure(&self) -> Structure {
//         match self {
//             StructureIndexed::Product(product) => StructureIndexed::Product(ProductStructure {
//                 values: product
//                     .values
//                     .iter()
//                     .map(|v| match v {
//                         StructureValue::Structure(index) => {
//                             StructureValue::Structure(force_unwrap_shared_index(index))
//                         }
//                         StructureValue::Token(token) => StructureValue::Token(*token),
//                         StructureValue::Group(group, index) => {
//                             StructureValue::Group(*group, force_unwrap_shared_index(index))
//                         }
//                     })
//                     .collect(),
//             }),
//             StructureIndexed::Sum(sum) => StructureIndexed::Sum(SumStructure {
//                 variants: sum.variants.iter().map(force_unwrap_shared_index).collect(),
//             }),
//         }
//     }
// }
