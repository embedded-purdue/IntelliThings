#[derive(Debug)]
pub enum SensorError {
    NoError,
    NotFound,
    InvalidData
}

/// T is the human readable formatted data type for each sensor
trait Sensor<T> {
    pub async fn poll() -> Result<self, SensorError>;
    fn convert(&self) -> T;
}
