#[macro_export]
macro_rules! cfg_steam { 
    ($($item:item)*) => {
        $(
            #[cfg(feature = "steam")]
            $item
        )*
    }
}

#[macro_export]
macro_rules! cfg_no_steam {
    ($($item:item)*) => {
        $(
            #[cfg(not(feature = "steam"))]
            $item
        )*
    }
}

#[macro_export]
macro_rules! cfg_audio {
    ($($item:item)*) => {
        $(
            #[cfg(feature = "audio")]
            $item
        )*
    } 
}

#[macro_export]
macro_rules! cfg_no_audio {
    ($($item:item)*) => {
        $(
            #[cfg(not(feature = "audio"))]
            $item
        )*
    } 
}

#[macro_export]
macro_rules! cfg_no_glfw {
    ($($item:item)*) => {
        $(
            #[cfg(not(feature = "glfw"))]
            $item
        )*
    }
}

#[macro_export]
macro_rules! cfg_glfw {
    ($($item:item)*) => {
        $(
            #[cfg(feature = "glfw")]
            $item
        )*
    }
}

#[macro_export]
macro_rules! path {
    ($path:literal) => {{
        #[cfg(not(feature = "steam"))] {
            concat!(env!("CARGO_MANIFEST_DIR"), "/../", $path)
        }
        #[cfg(feature = "steam")] {
            $path
        }
    }}
}
