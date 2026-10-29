use std::sync::Arc;

use asset_core::{AssetNamespace, AssetRef, WalkLocalMaterialIndex};
use asset_material::{AuthoredImage, AuthoredMaterial, MaterialDefinitions, TS_COLOR_MAP};
use asset_model::{
    BoneBind, ModelLodSelector, ModelSkel, VertSkin, dobj::ModelPoseSrc, pack_unit_vec,
};
use asset_world::{MapXModelAssetKey, MapXModelSceneAsset};
use bevy::asset::RenderAssetUsages;
use bevy::image::ImageSampler;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

use crate::PreparedWorld;

const MESH_MAGIC: &[u8; 8] = b"IW4LKC1\0";
const VERTEX_FLOATS: usize = 11;
const VERTEX_BYTES: usize = VERTEX_FLOATS * size_of::<f32>();
const MESH_BYTES: &[u8] = include_bytes!("../assets/kill_confirmed_dogtag/mesh.bin");

pub(crate) const FRIENDLY_MODEL: &str = "iw4l_kc_dogtag_friendly";
pub(crate) const ENEMY_MODEL: &str = "iw4l_kc_dogtag_enemy";
const FRIENDLY_TEXTURE: &[u8] = include_bytes!("../assets/kill_confirmed_dogtag/friendly.png");
const ENEMY_TEXTURE: &[u8] = include_bytes!("../assets/kill_confirmed_dogtag/enemy.png");

struct Mesh {
    vertices: Vec<[f32; VERTEX_FLOATS]>,
    indices: Vec<u32>,
    bounds_min: [f32; 3],
    bounds_max: [f32; 3],
}

fn read_u32(bytes: &[u8], offset: &mut usize) -> Result<u32, String> {
    let raw = bytes
        .get(*offset..*offset + 4)
        .ok_or_else(|| "truncated dog-tag mesh header".to_owned())?;
    *offset += 4;
    Ok(u32::from_le_bytes(raw.try_into().expect("four-byte slice")))
}

fn read_f32_array<const N: usize>(bytes: &[u8], offset: &mut usize) -> Result<[f32; N], String> {
    let mut values = [0.0; N];
    for value in &mut values {
        let raw = bytes
            .get(*offset..*offset + 4)
            .ok_or_else(|| "truncated dog-tag vertex".to_owned())?;
        *offset += 4;
        *value = f32::from_le_bytes(raw.try_into().expect("four-byte slice"));
    }
    Ok(values)
}

fn parse_mesh(bytes: &[u8]) -> Result<Mesh, String> {
    if bytes.get(..8) != Some(MESH_MAGIC.as_slice()) {
        return Err("dog-tag mesh has an unknown format".to_owned());
    }
    let mut offset = 8;
    let vertex_count = read_u32(bytes, &mut offset)? as usize;
    let index_count = read_u32(bytes, &mut offset)? as usize;
    if vertex_count == 0 || index_count == 0 || index_count % 3 != 0 {
        return Err(format!(
            "dog-tag mesh has invalid counts: {vertex_count} vertices, {index_count} indices"
        ));
    }

    let vertices_end = offset
        .checked_add(
            vertex_count
                .checked_mul(VERTEX_BYTES)
                .ok_or("dog-tag mesh overflow")?,
        )
        .ok_or("dog-tag mesh overflow")?;
    let indices_end = vertices_end
        .checked_add(index_count.checked_mul(4).ok_or("dog-tag mesh overflow")?)
        .ok_or("dog-tag mesh overflow")?;
    if bytes.len() != indices_end {
        return Err(format!(
            "dog-tag mesh length mismatch: expected {indices_end}, got {}",
            bytes.len()
        ));
    }

    let mut vertices = Vec::with_capacity(vertex_count);
    let mut bounds_min = [f32::INFINITY; 3];
    let mut bounds_max = [f32::NEG_INFINITY; 3];
    for _ in 0..vertex_count {
        let vertex: [f32; VERTEX_FLOATS] = read_f32_array(bytes, &mut offset)?;
        if !vertex.iter().all(|value| value.is_finite()) {
            return Err("dog-tag mesh contains a non-finite vertex value".to_owned());
        }
        for axis in 0..3 {
            bounds_min[axis] = bounds_min[axis].min(vertex[axis]);
            bounds_max[axis] = bounds_max[axis].max(vertex[axis]);
        }
        vertices.push(vertex);
    }
    let mut indices = Vec::with_capacity(index_count);
    for _ in 0..index_count {
        let index = read_u32(bytes, &mut offset)?;
        if index as usize >= vertex_count {
            return Err(format!(
                "dog-tag mesh index {index} exceeds {vertex_count} vertices"
            ));
        }
        indices.push(index);
    }
    Ok(Mesh {
        vertices,
        indices,
        bounds_min,
        bounds_max,
    })
}

fn f32_to_half_bits(value: f32) -> u16 {
    let bits = value.to_bits();
    let sign = ((bits >> 16) & 0x8000) as u16;
    let exponent = ((bits >> 23) & 0xff) as i32 - 127 + 15;
    let mantissa = bits & 0x7f_ffff;
    if exponent <= 0 {
        if exponent < -10 {
            return sign;
        }
        let mantissa = (mantissa | 0x80_0000) >> (1 - exponent);
        return sign | ((mantissa >> 13) as u16);
    }
    if exponent >= 0x1f {
        return sign | 0x7c00;
    }
    sign | ((exponent as u16) << 10) | ((mantissa >> 13) as u16)
}

fn packed_vertex(vertex: &[f32; VERTEX_FLOATS]) -> [u8; asset_iw4::size::GFX_PACKED_VERTEX] {
    let mut packed = [0u8; asset_iw4::size::GFX_PACKED_VERTEX];
    for (index, value) in vertex[..3].iter().enumerate() {
        packed[index * 4..index * 4 + 4].copy_from_slice(&value.to_le_bytes());
    }
    packed[16..20].copy_from_slice(&[255; 4]);
    let uv =
        (u32::from(f32_to_half_bits(vertex[9])) << 16) | u32::from(f32_to_half_bits(vertex[10]));
    packed[20..24].copy_from_slice(&uv.to_le_bytes());
    packed[24..28].copy_from_slice(&pack_unit_vec([vertex[3], vertex[4], vertex[5]]).to_le_bytes());
    packed[28..32].copy_from_slice(&pack_unit_vec([vertex[6], vertex[7], vertex[8]]).to_le_bytes());
    packed
}

fn model_from_mesh(name: &str, mesh: Mesh, material: usize) -> ModelSkel {
    let center = [
        (mesh.bounds_min[0] + mesh.bounds_max[0]) * 0.5,
        (mesh.bounds_min[1] + mesh.bounds_max[1]) * 0.5,
        (mesh.bounds_min[2] + mesh.bounds_max[2]) * 0.5,
    ];
    let half = [
        (mesh.bounds_max[0] - mesh.bounds_min[0]) * 0.5,
        (mesh.bounds_max[1] - mesh.bounds_min[1]) * 0.5,
        (mesh.bounds_max[2] - mesh.bounds_min[2]) * 0.5,
    ];
    let radius = (half[0] * half[0] + half[1] * half[1] + half[2] * half[2]).sqrt();
    let vertex_count = mesh.vertices.len();
    let index_count = mesh.indices.len();
    let positions = mesh
        .vertices
        .iter()
        .map(|vertex| [vertex[0], vertex[1], vertex[2]])
        .collect::<Vec<_>>();
    let packed_vertices = mesh.vertices.iter().map(packed_vertex).collect::<Vec<_>>();

    ModelSkel {
        name: name.to_owned(),
        bones: vec![BoneBind {
            quat: [0.0, 0.0, 0.0, 1.0],
            trans: [0.0, 0.0, 0.0],
        }],
        bone_collision: Vec::new(),
        bone_names: vec!["tag_origin".to_owned()],
        tag_view: None,
        tag_weapon: None,
        pose: Some(ModelPoseSrc {
            name: name.to_owned(),
            num_bones: 1,
            num_root_bones: 1,
            scale: 1.0,
            no_scale_part_bits: [0; 6],
            bone_names: vec!["tag_origin".to_owned()],
            parent_list: Vec::new(),
            quats: Vec::new(),
            trans: Vec::new(),
            base_mat: vec![(Quat::IDENTITY, Vec3::ZERO)],
            root_rest: None,
        }),
        positions: positions.into(),
        indices: mesh.indices.into(),
        surface_materials: vec![Some(WalkLocalMaterialIndex::from_walk(material))],
        surface_vertex_ranges: vec![(0, vertex_count)],
        surface_index_ranges: vec![(0, index_count)],
        surface_part_bits: vec![[0x8000_0000, 0, 0, 0, 0, 0]],
        surface_deformed: vec![Some(false)],
        surface_vert_list_count: vec![Some(vertex_count as u32)],
        vert_skin: vec![VertSkin::rigid(0); vertex_count].into(),
        rigid_verts: vertex_count,
        blend_verts: 0,
        packed_vertices: packed_vertices.into(),
        radius: Some(radius),
        bounds: Some((center, half)),
        contents: Some(0),
        coll_lod: 0,
        coll_surfs: Vec::new(),
        movement_brushes: Vec::new(),
        mount_tag: None,
        lod: Some(ModelLodSelector::Iw4 {
            lod_start: 0,
            num_lods: 1,
            lod_dist: [100_000.0, 0.0, 0.0, 0.0],
        }),
        lod_smc: Some([[0; 4]; 4]),
        lod_part_bits: Some([[0x8000_0000, 0, 0, 0, 0, 0]; 4]),
        lod_surf_span: [(0, 1), (0, 0), (0, 0), (0, 0)],
    }
}

fn decode_texture(name: &str, bytes: &[u8]) -> Result<Arc<Image>, String> {
    let decoder = png::Decoder::new(std::io::Cursor::new(bytes));
    let mut reader = decoder
        .read_info()
        .map_err(|error| format!("{name}: PNG header: {error}"))?;
    let (width, height) = reader.info().size();
    if reader.info().color_type != png::ColorType::Rgba
        || reader.info().bit_depth != png::BitDepth::Eight
    {
        return Err(format!("{name}: expected an 8-bit RGBA PNG"));
    }
    let size = reader
        .output_buffer_size()
        .ok_or_else(|| format!("{name}: decoded image is too large"))?;
    let mut pixels = vec![0; size];
    let frame = reader
        .next_frame(&mut pixels)
        .map_err(|error| format!("{name}: PNG decode: {error}"))?;
    pixels.truncate(frame.buffer_size());
    let mut image = Image::new(
        Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        pixels,
        TextureFormat::Rgba8Unorm,
        RenderAssetUsages::RENDER_WORLD,
    );
    image.sampler = ImageSampler::linear();
    Ok(Arc::new(image))
}

fn simple_lit_template(materials: &MaterialDefinitions) -> Option<(AuthoredMaterial, usize)> {
    materials
        .materials
        .iter()
        .filter(|material| material.namespace == AssetNamespace::Iw4)
        .filter(|material| materials.takes_model_lighting(material) == Some(true))
        .filter(|material| materials.is_unlit(material) == Some(false))
        .filter_map(|material| {
            let binding =
                material.textures.iter().enumerate().find(|(_, texture)| {
                    texture.semantic == TS_COLOR_MAP && texture.image.is_some()
                })?;
            let bound_textures = material
                .textures
                .iter()
                .filter(|texture| texture.image.is_some())
                .count();
            Some((bound_textures, material.clone(), binding.0))
        })
        .min_by_key(|(bound_textures, _, _)| *bound_textures)
        .map(|(_, material, binding)| (material, binding))
}

fn install_material(
    team: &str,
    texture: &[u8],
    template: &(AuthoredMaterial, usize),
    materials: &mut MaterialDefinitions,
    map_ids: &mut Vec<Option<usize>>,
) -> Result<usize, String> {
    let name = format!("iw4l_kc_dogtag_{team}");
    let decoded = decode_texture(&name, texture)?;
    let width = u16::try_from(decoded.width()).map_err(|_| format!("{name}: width exceeds u16"))?;
    let height =
        u16::try_from(decoded.height()).map_err(|_| format!("{name}: height exceeds u16"))?;

    let source_image = template.0.textures[template.1]
        .image
        .and_then(|index| materials.images.get(index))
        .cloned();
    let mut image = source_image.unwrap_or_else(|| AuthoredImage {
        namespace: AssetNamespace::Iw4,
        name: AssetRef::Real(name.clone()),
        map_type: 3,
        semantic: TS_COLOR_MAP,
        category: 0,
        use_srgb_reads: false,
        width,
        height,
        depth: 1,
        level_count: 1,
        format: 0,
        payload: Arc::new(Vec::new()),
        decoded: None,
        common_owned: false,
        decoded_variant: None,
        decoded_by: None,
        pending_decode: None,
    });
    image.name = AssetRef::Real(name.clone());
    image.width = width;
    image.height = height;
    image.depth = 1;
    image.level_count = 1;
    image.format = 0;
    image.payload = Arc::new(Vec::new());
    image.decoded = Some(decoded);
    image.common_owned = false;
    image.decoded_variant = None;
    image.decoded_by = None;
    image.pending_decode = None;
    let image_index = materials.images.len();
    materials.images.push(image);

    let mut material = template.0.clone();
    material.name = AssetRef::Real(name);
    material.namespace = AssetNamespace::Iw4;
    for (index, texture) in material.textures.iter_mut().enumerate() {
        if index != template.1 {
            texture.image = None;
        }
    }
    material.textures[template.1].image = Some(image_index);
    let material_index = materials.materials.len();
    materials.materials.push(material);
    let walk_local = map_ids.len();
    map_ids.push(Some(material_index));
    Ok(walk_local)
}

pub(crate) fn install_kill_confirmed_dogtags(
    world: &mut PreparedWorld,
    materials: &mut MaterialDefinitions,
    map_ids: &mut Vec<Option<usize>>,
) -> Result<String, String> {
    if world
        .map_xmodel_scene_assets
        .get_name(FRIENDLY_MODEL)
        .is_some()
    {
        return Ok("Kill Confirmed dog tags already installed".to_owned());
    }
    let Some(template) = simple_lit_template(materials) else {
        return Err(
            "Kill Confirmed dog tags: no opaque, lit, single-color-map IW4 material to clone"
                .to_owned(),
        );
    };

    let friendly_material =
        install_material("friendly", FRIENDLY_TEXTURE, &template, materials, map_ids)?;
    let enemy_material = install_material("enemy", ENEMY_TEXTURE, &template, materials, map_ids)?;
    let friendly_mesh = parse_mesh(MESH_BYTES)?;
    let vertex_count = friendly_mesh.vertices.len();
    let index_count = friendly_mesh.indices.len();
    let friendly = Arc::new(model_from_mesh(
        FRIENDLY_MODEL,
        friendly_mesh,
        friendly_material,
    ));
    let enemy = Arc::new(model_from_mesh(
        ENEMY_MODEL,
        parse_mesh(MESH_BYTES)?,
        enemy_material,
    ));
    world.map_xmodel_scene_assets.insert(
        MapXModelAssetKey(FRIENDLY_MODEL.to_owned()),
        MapXModelSceneAsset::Iw4(friendly),
    );
    world.map_xmodel_scene_assets.insert(
        MapXModelAssetKey(ENEMY_MODEL.to_owned()),
        MapXModelSceneAsset::Iw4(enemy),
    );
    Ok(format!(
        "Kill Confirmed dog tags: project model {vertex_count} vertices / {} triangles, friendly+enemy materials, template iw4:`{}`",
        index_count / 3,
        template.0.name.as_str(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_mesh_has_expected_topology_and_bounds() {
        let mesh = parse_mesh(MESH_BYTES).expect("embedded dog-tag mesh");
        assert_eq!(mesh.vertices.len(), 3505);
        assert_eq!(mesh.indices.len(), 6412 * 3);
        assert_eq!(
            mesh.indices.iter().copied().max().expect("dog-tag indices") as usize,
            3504
        );

        let skel = model_from_mesh(FRIENDLY_MODEL, mesh, 0);
        let (center, half) = skel.bounds.expect("dog-tag bounds");
        assert!(center.iter().all(|value| value.abs() < 0.001));
        assert!((half[0] * 2.0 - 14.4).abs() < 0.01);
        assert!(skel.pose.is_some());
        assert_eq!(skel.pose.as_ref().expect("dog-tag pose").num_bones, 1);
        assert_eq!(skel.surface_vertex_ranges, [(0, 3505)]);
    }

    #[test]
    fn embedded_team_textures_decode_as_rgba8() {
        let friendly = decode_texture("friendly", FRIENDLY_TEXTURE).expect("friendly texture");
        let enemy = decode_texture("enemy", ENEMY_TEXTURE).expect("enemy texture");
        assert_eq!(friendly.width(), 512);
        assert_eq!(friendly.height(), 512);
        assert_eq!(
            friendly.texture_descriptor.format,
            TextureFormat::Rgba8Unorm
        );
        assert_ne!(friendly.data, enemy.data);
    }
}
