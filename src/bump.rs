pub struct BumpBox{
	slice:Box<[u8]>,
	position:usize,
}
impl BumpBox{
	pub fn new_uninit(capacity:usize)->Self{
		BumpBox{
			slice:unsafe{Box::new_uninit_slice(capacity).assume_init()},
			position:capacity,
		}
	}
}
