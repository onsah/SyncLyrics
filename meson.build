project(
    'com.github.onsah.SyncLyrics', 
    'rust',
    license: 'GPLv3',
)

synclyrics_prefix = get_option('prefix')
synclyrics_bindir = join_paths(synclyrics_prefix, get_option('bindir'))

cargo = find_program('cargo', required: false)
gresource = find_program('glib-compile-resources', required: false)
cargo_vendor = find_program('cargo-vendor', required: false)
cargo_script = find_program('meson/cargo.sh')

cargo_release = custom_target('cargo-build',
                        build_by_default: true,
                        build_always_stale: true,
                        output: [meson.project_name ()],
                        install: true,
                        install_dir: synclyrics_bindir,
                        # command: [cargo, 'build', '-p', 'sync_lyrics'])
                        command: [cargo_script, '@CURRENT_SOURCE_DIR@', '@OUTPUT@'],
                        console: true)

meson.add_install_script('meson/post_install.py')

subdir('data')