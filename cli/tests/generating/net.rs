use tolearn_offline::reach::Reach;

#[derive(Debug)]
pub struct Net(pub bool);

impl Reach for Net {
    fn reach(&self, _url: &str) -> Result<(), String> {
        match self.0 {
            true => Ok(()),
            false => Err("network is unreachable".to_owned()),
        }
    }
}
