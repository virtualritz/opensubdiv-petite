#[repr(u32)]
#[derive(Copy, Clone, Debug)]
pub enum VtxBoundaryInterpolation {
    None = 0,
    EdgeOnly,
    EdgeAndCorner,
}

#[repr(u32)]
#[derive(Copy, Clone, Debug)]
pub enum FVarLinearInterpolation {
    None = 0,
    CornersOnly,
    CornersPlus1,
    CornersPlus2,
    Boundaries,
    All,
}

#[repr(u32)]
#[derive(Copy, Clone, Debug)]
pub enum CreasingMethod {
    Uniform = 0,
    Chaikin,
}

#[repr(u32)]
#[derive(Copy, Clone, Debug)]
pub enum TriangleSubdivision {
    CatmullClark = 0,
    Smooth,
}

bitfield! {
    #[repr(C)] #[derive(Copy, Clone, PartialEq, Debug)] pub struct Options(u32);
    u32;
    pub vtx_bound_interp, set_vtx_bound_interp: 1, 0;
    pub fvar_lin_interp, set_fvar_lin_interp: 4, 2;
    pub creasing_method, set_creasing_method: 6, 5;
    pub triangle_sub, set_triangle_sub: 8, 7;
}

impl Default for Options {
    fn default() -> Options {
        let mut opt = Options(0);
        opt.set_fvar_lin_interp(FVarLinearInterpolation::All as u32);
        opt
    }
}

pub struct OptionsBuilder {
    vtx_bound_interp: VtxBoundaryInterpolation,
    fvar_lin_interp: FVarLinearInterpolation,
    creasing_method: CreasingMethod,
    triangle_sub: TriangleSubdivision,
}

impl OptionsBuilder {
    pub fn new() -> OptionsBuilder {
        OptionsBuilder {
            vtx_bound_interp: VtxBoundaryInterpolation::None,
            fvar_lin_interp: FVarLinearInterpolation::All,
            creasing_method: CreasingMethod::Uniform,
            triangle_sub: TriangleSubdivision::CatmullClark,
        }
    }

    pub fn vtx_boundary_interpolation<'a>(
        &'a mut self,
        vtx_bound_interp: VtxBoundaryInterpolation,
    ) -> &'a OptionsBuilder {
        self.vtx_bound_interp = vtx_bound_interp;
        self
    }

    pub fn fvar_linear_interpolation<'a>(
        &'a mut self,
        fvar_lin_interp: FVarLinearInterpolation,
    ) -> &'a OptionsBuilder {
        self.fvar_lin_interp = fvar_lin_interp;
        self
    }

    pub fn creasing_method<'a>(
        &'a mut self,
        creasing_method: CreasingMethod,
    ) -> &'a OptionsBuilder {
        self.creasing_method = creasing_method;
        self
    }

    pub fn triangle_subdivision<'a>(
        &'a mut self,
        triangle_sub: TriangleSubdivision,
    ) -> &'a OptionsBuilder {
        self.triangle_sub = triangle_sub;
        self
    }

    pub fn build(&self) -> Options {
        let mut opt = Options::default();
        opt.set_vtx_bound_interp(self.vtx_bound_interp as u32);
        opt.set_fvar_lin_interp(self.fvar_lin_interp as u32);
        opt.set_creasing_method(self.creasing_method as u32);
        opt.set_triangle_sub(self.triangle_sub as u32);
        opt
    }
}
