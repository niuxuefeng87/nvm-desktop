use crate::core::{node, project};
use crate::utils::resolve;
use crate::{cmds, config::Config, log_err};
use anyhow::{bail, Ok, Result};
use tauri::menu::{AboutMetadataBuilder, CheckMenuItem};
use tauri::tray::{MouseButton, TrayIconEvent};
use tauri::{
    async_runtime::spawn,
    image::Image,
    menu::{
        CheckMenuItemBuilder, IsMenuItem, Menu, MenuBuilder, MenuEvent, MenuItemBuilder,
        PredefinedMenuItem, Submenu, SubmenuBuilder,
    },
    AppHandle, Emitter, Manager, Wry,
};

use super::handle;

pub struct Tray {}

fn gen_check_menu_items(
    app_handle: &AppHandle,
    versions: &[String],
    name: &str,
    current: &str,
) -> Result<Vec<CheckMenuItem<Wry>>> {
    versions
        .iter()
        .map(|version| {
            Ok(CheckMenuItemBuilder::with_id(
                format!("{}_version_{}", name, version),
                format!("v{}", version),
            )
            .checked(current == version)
            .build(app_handle)?)
        })
        .collect()
}

impl Tray {
    pub fn create_tray_menu(app_handle: &AppHandle) -> Result<Menu<Wry>> {
        let package_info = app_handle.package_info();
        let version = package_info.version.to_string();
        let app_name: String = package_info.name.to_string();
        let zh = { Config::settings().latest().locale == Some("zh-CN".into()) };
        // projects (keep max length 5)
        let mut projects = { Config::projects().latest().get_list() }.unwrap_or(vec![]);
        projects.truncate(5);
        // groups
        let groups = Config::groups();
        let groups = groups.latest();
        let groups = groups.list.as_deref().unwrap_or(&[]);
        // installed versions
        let node = Config::node();
        let node = node.latest();
        let installed = node.installed.as_deref().unwrap_or(&[]);
        let global_current = node.current.as_deref().unwrap_or_default();

        let icon_path = app_handle.path().resource_dir()?.join("icons/icon.png");

        macro_rules! t {
            ($en: expr, $zh: expr) => {
                if zh {
                    $zh
                } else {
                    $en
                }
            };
        }

        let sub_items = projects
            .iter()
            .map(|project| {
                let project_version = project.version.as_deref().unwrap_or("");
                let version_items =
                    gen_check_menu_items(app_handle, &installed, &project.name, project_version)?;
                let group_items = groups
                    .iter()
                    .map(|group| {
                        Ok(CheckMenuItemBuilder::new(&group.name)
                            .id(format!("{}_group_{}", &project.name, &group.name))
                            .checked(project_version == &group.name)
                            .build(app_handle)?)
                    })
                    .collect::<Result<Vec<_>>>()?;
                let version_items_refs: Vec<&dyn IsMenuItem<Wry>> = version_items
                    .iter()
                    .map(|item| item as &dyn IsMenuItem<Wry>)
                    .collect();
                let group_items_refs: Vec<&dyn IsMenuItem<Wry>> = group_items
                    .iter()
                    .map(|item| item as &dyn IsMenuItem<Wry>)
                    .collect();

                Ok(
                    SubmenuBuilder::with_id(app_handle, &project.name, &project.name)
                        .items(&version_items_refs)
                        .separator()
                        .items(&group_items_refs)
                        .build()?,
                )
            })
            .collect::<Result<Vec<Submenu<Wry>>>>()?;
        let sub_items_refs: Vec<&dyn IsMenuItem<Wry>> = sub_items
            .iter()
            .map(|item| item as &dyn IsMenuItem<Wry>)
            .collect();

        let global_menu_items =
            gen_check_menu_items(app_handle, &installed, "global", global_current)?;
        let global_menu_items_ref: Vec<&dyn IsMenuItem<Wry>> = global_menu_items
            .iter()
            .map(|item| item as &dyn IsMenuItem<Wry>)
            .collect();

        Ok(MenuBuilder::with_id(app_handle, "tray_menu")
            .item(
                &MenuItemBuilder::with_id("open_window", t!("NVM-Desktop", "NVM-Desktop"))
                    .build(app_handle)?,
            )
            .separator()
            .item(
                &SubmenuBuilder::with_id(app_handle, "global", "Global (defaule)")
                    .items(&global_menu_items_ref)
                    .build()?,
            )
            .items(&sub_items_refs)
            .separator()
            .items(&[
                &SubmenuBuilder::with_id(app_handle, "open_dirs", t!("Open Dir", "打开目录"))
                    .items(&[
                        &MenuItemBuilder::with_id("open_data_dir", t!("Data Dir", "数据目录"))
                            .build(app_handle)?,
                        &MenuItemBuilder::with_id("open_logs_dir", t!("Logs Dir", "日志目录"))
                            .build(app_handle)?,
                    ])
                    .build()?,
                &MenuItemBuilder::with_id("open_dev_tools", t!("Open Dev Tools", "开发者工具"))
                    .build(app_handle)?,
                &PredefinedMenuItem::about(
                    app_handle,
                    Some(t!("About NVM-Desktop", "关于 NVM-Desktop")),
                    Some(
                        AboutMetadataBuilder::new()
                            .version(Some(version))
                            .name(Some(app_name))
                            .icon(Some(Image::from_path(icon_path)?))
                            .build(),
                    ),
                )?,
                &MenuItemBuilder::with_id("quit", t!("Quit NVM-Desktop", "退出 NVM-Desktop"))
                    .accelerator("CmdOrCtrl+Q")
                    .build(app_handle)?,
            ])
            .build()?)
    }

    pub fn create_systray() -> Result<()> {
        let app_handle = handle::Handle::global().app_handle().unwrap();
        if let Some(tray) = app_handle.tray_by_id("main") {
            tray.on_tray_icon_event(|_, event| {
                #[cfg(not(target_os = "macos"))]
                if let TrayIconEvent::Click {
                    button: MouseButton::Left,
                    ..
                } = event
                {
                    let _ = resolve::create_window();
                }
            });
            tray.on_menu_event(Tray::on_menu_event);
        }

        Ok(())
    }

    pub fn update_part() -> Result<()> {
        let app_handle = handle::Handle::global().app_handle().unwrap();
        if let Some(tray) = app_handle.tray_by_id("main") {
            let _ = tray.set_menu(Some(Tray::create_tray_menu(&app_handle)?));
            Ok(())
        } else {
            bail!("The system tray menu has not been initialized")
        }
    }

    pub fn update_part_with_emit(event: &str, version: &str) -> Result<()> {
        let app_handle = handle::Handle::global().app_handle().unwrap();
        if let Some(tray) = app_handle.tray_by_id("main") {
            let _ = tray.set_menu(Some(Tray::create_tray_menu(&app_handle)?));
            if let Some(window) = handle::Handle::global().get_window() {
                window.emit(event, version)?;
            }
            Ok(())
        } else {
            bail!("The system tray menu has not been initialized")
        }
    }

    pub fn on_menu_event(app_handle: &AppHandle, event: MenuEvent) {
        match event.id().as_ref() {
            "open_window" => {
                let _ = resolve::create_window();
            }
            "quit" => cmds::exit_app(app_handle.clone()),
            "open_data_dir" => crate::log_err!(cmds::open_data_dir()),
            "open_logs_dir" => crate::log_err!(cmds::open_logs_dir()),
            "open_dev_tools" => {
                if let Some(window) = app_handle.get_webview_window("main") {
                    window.open_devtools();
                }
            }
            id if id.contains("_version_") => Tray::handle_version_change(id),
            id if id.contains("_group_") => Tray::handle_group_change(id),
            _ => {}
        }
    }

    fn handle_version_change(id: &str) {
        let info = id.split("_version_").collect::<Vec<_>>();
        if info.len() == 2 {
            let name = info[0].to_owned();
            let version = info[1].to_owned();
            if name == "global" {
                spawn(async move {
                    log_err!(node::update_current_from_menu(version).await);
                });
            } else {
                spawn(async move {
                    log_err!(project::change_with_version(name, version).await);
                });
            }
        }
    }

    fn handle_group_change(id: &str) {
        let info = id.split("_group_").collect::<Vec<_>>();
        if info.len() == 2 {
            let name = info[0].to_owned();
            let group_name = info[1].to_owned();
            spawn(async move {
                log_err!(project::change_with_group(name, group_name).await);
            });
        }
    }
}
