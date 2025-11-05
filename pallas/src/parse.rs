pub struct Name{
  pub name: String
}

pub struct AST{
  pub names: OrderSet<Name>
} impl AST{
  pub fn new() -> Self
  pub fn register_name(&mut self, name: Name) -> Result<usize, ()>{
    let (idx, success) = self.names.insert_full(name);
    if success{
      Ok(idx)
    } else{
      Err(())
    }
  }
}