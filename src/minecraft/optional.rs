use bytes::BufMut;

use crate::minecraft::packet::WritePacketData;

impl<T: WritePacketData> WritePacketData for Option<T> {
    fn write(self, data: &mut bytes::BytesMut) {
        if let Some(val) = self {
            data.put_u8(1);
            val.write(data);
        } else {
            data.put_u8(0);
        }
    }
}
