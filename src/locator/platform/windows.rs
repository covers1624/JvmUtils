use crate::install::JavaInstall;
use crate::locator::platform::PlatformJavaLocator;
use crate::locator::{find_add_install, scan_folder, JavaLocator};
use crate::log_debug;
use winreg::enums::HKEY_LOCAL_MACHINE;
use winreg::RegKey;

// Oracle.
const ORACLE: &'static [&'static str] = &[
    /*HKEY_LOCAL_MACHINE\\*/ "SOFTWARE\\JavaSoft\\Java Development Kit",
    /*HKEY_LOCAL_MACHINE\\*/ "SOFTWARE\\JavaSoft\\Java Runtime Environment",
    /*HKEY_LOCAL_MACHINE\\*/ "SOFTWARE\\JavaSoft\\JRE",
    /*HKEY_LOCAL_MACHINE\\*/ "SOFTWARE\\JavaSoft\\JDK",
    /*HKEY_LOCAL_MACHINE\\*/ "SOFTWARE\\Wow6432Node\\JavaSoft\\Java Development Kit",
    /*HKEY_LOCAL_MACHINE\\*/ "SOFTWARE\\Wow6432Node\\JavaSoft\\Java Runtime Environment",
    /*HKEY_LOCAL_MACHINE\\*/ "SOFTWARE\\Wow6432Node\\JavaSoft\\JRE",
    /*HKEY_LOCAL_MACHINE\\*/ "SOFTWARE\\Wow6432Node\\JavaSoft\\JDK",
];

// AdoptOpenJDK, Adoptium predecessor (OpenJDK).
const ADOPT_OPEN_JDK: &'static [&'static str] = &[
    /*HKEY_LOCAL_MACHINE\\*/ "SOFTWARE\\AdoptOpenJDK\\JDK",
    /*HKEY_LOCAL_MACHINE\\*/ "SOFTWARE\\AdoptOpenJDK\\JRE",
    /*HKEY_LOCAL_MACHINE\\*/ "SOFTWARE\\Wow6432Node\\AdoptOpenJDK\\JDK",
    /*HKEY_LOCAL_MACHINE\\*/ "SOFTWARE\\Wow6432Node\\AdoptOpenJDK\\JRE",
];

// Adoptium, AdoptOpenJDK successor (OpenJDK).
const ADOPTIUM: &'static [&'static str] = &[
    /*HKEY_LOCAL_MACHINE\\*/ "SOFTWARE\\Eclipse Foundation\\JDK",
    /*HKEY_LOCAL_MACHINE\\*/ "SOFTWARE\\Eclipse Foundation\\JRE",
    /*HKEY_LOCAL_MACHINE\\*/ "SOFTWARE\\Wow6432Node\\Eclipse Foundation\\JDK",
    /*HKEY_LOCAL_MACHINE\\*/ "SOFTWARE\\Wow6432Node\\Eclipse Foundation\\JRE",
    /*HKEY_LOCAL_MACHINE\\*/ "SOFTWARE\\Eclipse Adoptium\\JDK",
    /*HKEY_LOCAL_MACHINE\\*/ "SOFTWARE\\Eclipse Adoptium\\JRE",
    /*HKEY_LOCAL_MACHINE\\*/ "SOFTWARE\\Wow6432Node\\Eclipse Adoptium\\JDK",
    /*HKEY_LOCAL_MACHINE\\*/ "SOFTWARE\\Wow6432Node\\Eclipse Adoptium\\JRE",
];

// Microsoft (OpenJDK).
const MICROSOFT: &'static [&'static str] = &[
    /*HKEY_LOCAL_MACHINE\\*/ "SOFTWARE\\Microsoft\\JDK",
    /*HKEY_LOCAL_MACHINE\\*/ "SOFTWARE\\Microsoft\\JRE",
    /*HKEY_LOCAL_MACHINE\\*/ "SOFTWARE\\Wow6432Node\\Microsoft\\JDK",
    /*HKEY_LOCAL_MACHINE\\*/ "SOFTWARE\\Wow6432Node\\Microsoft\\JRE",
];

// Common disk locations.
const PATHS: &'static [&'static str] = &[
    "C:/Program Files/AdoptOpenJDK/",
    "C:/Program Files/Eclipse Foundation/",
    "C:/Program Files/Eclipse Adoptium/",
    "C:/Program Files/Java/",
    "C:/Program Files/Microsoft/",
    "C:/Program Files (x86)/AdoptOpenJDK/",
    "C:/Program Files (x86)/Eclipse Foundation/",
    "C:/Program Files (x86)/Eclipse Adoptium/",
    "C:/Program Files (x86)/Java",
    "C:/Program Files (x86)/Microsoft/",
];

fn get_sub_keys(hklm: &RegKey, key: &str) -> Vec<String> {
    if let Ok(opened) = hklm.open_subkey(key) {
        return opened.enum_keys()
            .filter_map(|e| e.ok())
            .map(|e| key.to_owned() + "\\" + &e)
            .collect();
    }
    Vec::new()
}

fn scan_registry(mut vec: &mut Vec<JavaInstall>, keys: impl IntoIterator<Item=impl AsRef<str>>, key_suffix: &str, path_key: &str) {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    keys.into_iter()
        .flat_map(|k| get_sub_keys(&hklm, k.as_ref()))
        .filter_map(|e| hklm.open_subkey(e + "\\" + key_suffix).ok())
        .filter_map(|e| e.get_value::<String, _>(path_key).ok())
        .for_each(|e| { find_add_install(&mut vec, &e); });
}

impl JavaLocator for PlatformJavaLocator {
    fn locate(&self) -> Option<Vec<JavaInstall>> {
        let mut vec: Vec<JavaInstall> = Vec::new();
        // Search known registry keys.
        log_debug!("Searching for JVM's installed in common system registry locations.");
        scan_registry(&mut vec, ORACLE, "", "JavaHome");
        scan_registry(&mut vec, ADOPT_OPEN_JDK, "hotspot\\MSI", "Path");
        scan_registry(&mut vec, ADOPTIUM, "hotspot\\MSI", "Path");
        scan_registry(&mut vec, MICROSOFT, "hotspot\\MSI", "Path");

        // Try again in known paths.
        log_debug!("Searching for JVM's installed in common system locations.");
        for path in PATHS {
            scan_folder(&mut vec, path);
        }

        Some(vec)
    }
}
