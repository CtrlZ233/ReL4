use crate::config::*;

#[derive(PartialEq, Eq, Ord, PartialOrd, Copy, Clone, Debug)]
pub enum ObjectType {
    UntypedObject = 0,
    TCBObject = 1,
    EndpointObject = 2,
    NotificationObject = 3,
    CapTableObject = 4,
    NonArchObjectTypeCount = 5,
    Riscv4kpage = 6,
    RiscvMegaPage = 7,
    RiscvPageTableObject = 8,
    ObjectTypeCount = 9,
}

impl ObjectType {
    pub fn from_usize(t: usize) -> Self {
        unsafe {
            core::mem::transmute::<u8, ObjectType>(t as u8)
        }
    }

    pub fn is_frame_type(&self) -> bool {
        match self {
            Self::Riscv4kpage | Self::RiscvMegaPage => {
                true
            }
            _ => {
                false
            }
        }
    }

    pub fn get_size(&self, user_object_size: usize) -> usize {
        match *self {
            ObjectType::UntypedObject => user_object_size,
            ObjectType::TCBObject => SEL4_TCB_BITS,
            ObjectType::EndpointObject => SEL4_ENDPOINT_BITS,
            ObjectType::NotificationObject => SEL4_NOTIFICATION_BITS,
            ObjectType::CapTableObject => SEL4_SLOT_BITS + user_object_size,
            ObjectType::Riscv4kpage | ObjectType::RiscvPageTableObject => PAGE_BITS,
            _ => {
                // error!("invalid object type: {}", t as usize);
                return 0;
            }
        }
    }
}

pub fn get_object_size(t: ObjectType, user_object_size: usize) -> usize {
    t.get_size(user_object_size)
}
