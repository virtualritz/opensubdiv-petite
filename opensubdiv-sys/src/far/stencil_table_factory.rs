use super::stencil_table::StencilTablePtr;
use super::topology_refiner::TopologyRefinerPtr;

#[repr(u32)]
pub enum InterpolationMode {
    Vertex = 0,
    Varying,
    FaceVarying,
}

bitfield! {
    #[repr(C)] #[derive(Copy, Clone)] pub struct OptionsFields(u32);
    u32;
    pub interpolation_mode, set_interpolation_mode: 1, 0;
    pub generate_offsets, set_generate_offsets: 2;
    pub generate_control_verts, set_generate_control_verts: 3;
    pub generate_intermediate_levels, set_generate_intermediate_levels: 4;
    pub factorize_intermediate_levels, set_factorize_intermediate_levels: 5;
    pub max_level, set_max_level: 9, 6;
}

#[repr(C)]
#[derive(Default)]
pub struct Options {
    pub fields: OptionsFields,
    pub fvar_channel: u32,
}

impl Default for OptionsFields {
    fn default() -> OptionsFields {
        let mut opt = OptionsFields(0);
        opt.set_interpolation_mode(InterpolationMode::Vertex as u32);
        opt.set_generate_offsets(false);
        opt.set_generate_control_verts(false);
        opt.set_generate_intermediate_levels(true);
        opt.set_factorize_intermediate_levels(true);
        opt.set_max_level(10);
        opt
    }
}

pub struct OptionsBuilder {
    interpolation_mode: InterpolationMode,
    generate_offsets: bool,
    generate_control_verts: bool,
    generate_intermediate_levels: bool,
    factorize_intermediate_levels: bool,
    max_level: u32,
    fvar_channel: u32,
}

impl OptionsBuilder {
    pub fn new() -> OptionsBuilder {
        OptionsBuilder {
            interpolation_mode: InterpolationMode::Vertex,
            generate_offsets: false,
            generate_control_verts: false,
            generate_intermediate_levels: true,
            factorize_intermediate_levels: true,
            max_level: 10,
            fvar_channel: 0,
        }
    }

    pub fn interpolation_mode(
        mut self,
        interpolation_mode: InterpolationMode,
    ) -> Self {
        self.interpolation_mode = interpolation_mode;
        self
    }

    pub fn generate_offsets(mut self, generate_offsets: bool) -> Self {
        self.generate_offsets = generate_offsets;
        self
    }

    pub fn generate_control_verts(
        mut self,
        generate_control_verts: bool,
    ) -> Self {
        self.generate_control_verts = generate_control_verts;
        self
    }

    pub fn generate_intermediate_levels(
        mut self,
        generate_intermediate_levels: bool,
    ) -> Self {
        self.generate_intermediate_levels = generate_intermediate_levels;
        self
    }

    pub fn factorize_intermediate_levels(
        mut self,
        factorize_intermediate_levels: bool,
    ) -> Self {
        self.factorize_intermediate_levels = factorize_intermediate_levels;
        self
    }

    pub fn max_level(mut self, max_level: u32) -> Self {
        self.max_level = max_level;
        self
    }

    pub fn build(self) -> Options {
        let mut fields = OptionsFields(0);
        fields.set_interpolation_mode(self.interpolation_mode as u32);
        fields.set_generate_offsets(self.generate_offsets);
        fields.set_generate_control_verts(self.generate_control_verts);
        fields.set_generate_intermediate_levels(
            self.generate_intermediate_levels,
        );
        fields.set_factorize_intermediate_levels(
            self.factorize_intermediate_levels,
        );
        fields.set_max_level(self.max_level);
        Options {
            fields,
            fvar_channel: self.fvar_channel,
        }
    }
}

extern "C" {
    pub fn StencilTableFactory_Create(
        refiner: TopologyRefinerPtr,
        options: Options,
    ) -> StencilTablePtr;

}
