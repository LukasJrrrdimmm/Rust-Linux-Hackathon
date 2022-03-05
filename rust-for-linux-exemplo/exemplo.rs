//! Exemplo simplificado
use kernel::prelude::*;
use kernel::file::File;
use kernel::file_operations::FileOperations;
use kernel::io_buffer::{IoBufferReader, IoBufferWriter};
use kernel::miscdev;
use kernel::sync::{Mutex, Ref, RefBorrow, UniqueRef};
module! {
	type: Exemplo,
	name: b"exemplo",
	license: b"GPL v2",
	params:{
		nr_devs: i32{
			default: 1,
			permissions:
		}
	}
}
struct Content{
	dados: Vec<u8>,
}
struct Exemplo{
	_dev: Pin<Box<miscdev::Registration<Exemplo>>>,
}
struct Dispositivo{
	number: usize,
	content: Mutex<Content>,
}

impl Dispositivo {
	fn try_new (number: usize) -> Result<Ref<Self>>{
		pr_info!(">>>>>>>>>> Safety :: mutex init\n");
		let mut disp = Pin::from(UniqueRef::try_new(Dispositivo{
			number,
			content: unsafe {
				Mutex::new(
					Content{
						dados: Vec::new()
					}
				)
			}
		})?);
		pr_info!(">>>>>>>>>> Safety :: conteudo\n");
		let m = unsafe{
			disp.as_mut().map_unchecked_mut(|d| &mut d.content)
		};
		kernel::mutex_init!(m, "Dispositivo::content");
		pr_info!("Oppening App!!!\n");
		Ok(disp.into())
	}
}

impl FileOperations for Exemplo {
	type OpenData = Ref<Dispositivo>;
	type Wrapper = Ref<Dispositivo>;

	kernel::declare_file_operations!(read, write);
	fn open(
		data:&Ref<Dispositivo>,
		_file:&File
	) -> Result<Ref<Dispositivo>>{
		pr_info!("Openning File {}!!!\n", data.number);
		Ok(data.clone())
	}
	fn read(
		this: RefBorrow<'_, Dispositivo>,
		_file: &File,
		data: &mut impl IoBufferWriter,
		offset:u64
	) -> Result<usize>{
		let offset = offset.try_into()?;
		let guard = this.content.lock();
		let len = core::cmp::min(
			data.len(),
			guard.dados.len().saturating_sub(offset)
		);
		pr_info!("Escrevendo.....\n");
		Ok(0)
	}
	fn write(
		this: RefBorrow<'_,Dispositivo>, 
		_file: &File,
		data: &mut impl IoBufferReader,
		_offset:u64
	) -> Result<usize>{
		pr_info!("Eu escrevi {} bytes\n", data.len());
		let copia = data.read_all()?;
		let len = data.len();
		this.content.lock().dados = copia;
		Ok(len)
	}
}

//comando quando começa
impl KernelModule for Exemplo{
	fn init(
		name: &'static CStr,
		module: &'static ThisModule
	) -> Result<Self> {
		pr_info!("Hello Wolrd!!!!!!!!!!!!!!!!!!\n");
		let count = {
			let lock = module.kernel_param_lock();
			(*nr_devs.read(&lock)).try_into()?
		}
		pr_info!("Hello Module {}, with devs = {}", name, module)
		let reg = miscdev::Registration::new_pinned(fmt!("exemplo"), Dispositivo::try_new(0)?)?;
		Ok(Exemplo{_dev: reg})
		// Err(Error::ENOMEM)
	} 
}

//comando quando cai
impl Drop for Exemplo{
	fn drop(&mut self){
		pr_info!("Bye Bye!!!\n");
	}
}
