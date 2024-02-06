use prusti_contracts::*;
#[extern_spec]
impl<T: Default> Option<T> {
    #[pure]
    #[ensures(result == matches!(self, None))]
    fn is_none(&self) -> bool;

    #[pure]
    #[ensures(result == matches!(self, Some(_)))]
    fn is_some(&self) -> bool;

    #[pure]
    #[requires(self.is_some())]
    #[ensures(self === Some(result))]
    fn unwrap(self) -> T;
}
