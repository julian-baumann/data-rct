// From https://github.com/lpxxn/rust-design-pattern/blob/master/behavioral/observer.rs
//! Observer is a behavioral design pattern that allows one objects to notify other objects about changes in their state.

pub trait IObserver {
    fn update(&self);
}

pub trait ISubject<'a, T: IObserver> {
    fn attach(&mut self, observer: &'a T);
    fn detach(&mut self, observer: &'a T);
    fn notify_observers(&self);
}

// struct Subject<'a, T: IObserver> {
//     observers: Vec<&'a T>,
// }
// impl<'a, T: IObserver + PartialEq> Subject<'a, T> {
//     fn new() -> Subject<'a, T> {
//         Subject {
//             observers: Vec::new(),
//         }
//     }
// }
//
// impl<'a, T: IObserver + PartialEq> ISubject<'a, T> for Subject<'a, T> {
//     fn attach(&mut self, observer: &'a T) {
//         self.observers.push(observer);
//     }
//     fn detach(&mut self, observer: &'a T) {
//         if let Some(idx) = self.observers.iter().position(|x| *x == observer) {
//             self.observers.remove(idx);
//         }
//     }
//     fn notify_observers(&self) {
//         for item in self.observers.iter() {
//             item.update();
//         }
//     }
// }