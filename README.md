# Distant Garden


### Just use cargo 

Client:
`cargo build -p voxelland-client --release`

Server 
`cargo build -p voxelland-server --release`

Must use release mode, will not run fast enough in debug mode.

For maximum optimizations:

Client:
`cargo build -p voxelland-client --profile deploy`

Server 
`cargo build -p voxelland-server --profile deploy`

### Dependencies
Glfw for all platforms
X11 and alsa libs for linux 
