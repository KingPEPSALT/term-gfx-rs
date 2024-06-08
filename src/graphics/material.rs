use super::LightColour;


#[derive(Debug, Clone)]
pub struct Material {
    /// Specular exponent
    pub specular: Option<f64>,
    /// Colour of the material
    pub colour: LightColour
}

