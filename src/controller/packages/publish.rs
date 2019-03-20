use std::convert::TryFrom;
use std::io::Read;
use std::path::Path;
use std::str::FromStr;

use actix_web::*;
use elba::package::manifest::{DepReq, Manifest};
use failure::Error;
use tar::Archive;
use tokio_async_await::await;

use crate::model::packages::*;
use crate::util::error::Reason;
use crate::{AppState, CONFIG};

use super::PackageVersionReq;

#[derive(Deserialize, Clone)]
pub struct PublishReq {
    pub token: String,
}

pub async fn publish(
    (path, query, state, req): (
        actix_web::Path<PackageVersionReq>,
        Query<PublishReq>,
        State<AppState>,
        HttpRequest<AppState>,
    ),
) -> Result<HttpResponse, Error> {
    let package_version = PackageVersion::try_from(path.into_inner())?;

    info!("Receiving tarball");

    let body = await!(req.body().limit(CONFIG.max_upload_size))?;

    let manifest = read_manifest(&body)?;
    verify_manifest(&package_version, &manifest)?;

    let deps = deps_in_manifest(&manifest)?;
    let readme_file = read_readme(
        &body,
        manifest
            .package
            .readme
            .as_ref()
            .map(|subpath| subpath.0.as_path()),
    )?;

    await!(state.db.send(PublishVersion {
        package: package_version.clone(),
        package_info: manifest.package.clone(),
        readme_file,
        dependencies: deps.clone(),
        token: query.token.clone(),
        bytes: body,
    }))??;

    Ok(HttpResponse::Ok().finish())
}

fn read_manifest(bytes: &[u8]) -> Result<Manifest, Error> {
    let mut archive = Archive::new(bytes);
    // TODO: Find manifest case-insensitively
    let mut entry = archive
        .entries()?
        .filter_map(Result::ok)
        .find(|entry| match entry.path() {
            Ok(ref path) if *path == Path::new("elba.toml") => true,
            _ => false,
        }).ok_or_else(|| human!(Reason::InvalidManifest, "Manifest not found in archive"))?;

    let mut buffer = String::new();
    entry.read_to_string(&mut buffer)?;
    let manifest = Manifest::from_str(&buffer)?;

    Ok(manifest)
}

fn read_readme(bytes: &[u8], subpath: Option<&Path>) -> Result<Option<String>, Error> {
    let mut archive = Archive::new(bytes);
    let entry = archive.entries()?.filter_map(Result::ok).find(|entry| {
        if let Ok(path) = entry.path() {
            if let Some(subpath) = subpath {
                if path == subpath {
                    return true;
                }
            }
            if let Some(file_stem) = path.file_stem() {
                return &file_stem.to_string_lossy().to_uppercase() == "README";
            }
            false
        } else {
            false
        }
    });

    if let Some(mut entry) = entry {
        if entry.header().entry_size()? > 2 * 1024 * 1024 {
            return Ok(None);
        }

        let mut buffer = String::new();
        entry.read_to_string(&mut buffer)?;

        Ok(Some(buffer))
    } else {
        Ok(None)
    }
}

fn verify_manifest(req: &PackageVersion, manifest: &Manifest) -> Result<(), Error> {
    if manifest.package.name.group() != req.name.group() {
        return Err(human!(
            Reason::InvalidManifest,
            "Package group name mismatched"
        ));
    }

    if manifest.package.name.name() != req.name.name() {
        return Err(human!(Reason::InvalidManifest, "Package name mismatched"));
    }

    if manifest.package.version != req.semver {
        return Err(human!(
            Reason::InvalidManifest,
            "Package version mismatched"
        ));
    }

    Ok(())
}

fn deps_in_manifest(manifest: &Manifest) -> Result<Vec<(DependencyReq)>, Error> {
    let mut deps = Vec::new();

    for (name, ver_req) in manifest.dependencies.iter() {
        let version_req = match ver_req {
            DepReq::Registry(constrain) => constrain.clone(),
            _ => {
                return Err(human!(
                    Reason::InvalidManifest,
                    "Package contains non-index dependency {}",
                    name.as_str()
                ))
            }
        };

        deps.push(DependencyReq {
            name: name.clone(),
            version_req,
        });
    }

    Ok(deps)
}
