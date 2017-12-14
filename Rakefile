# things this rakefile should do:
# 1. use nasm to compile .asm files to .o files
# 2. use ld to link the .o files
# 3. use grub-mkrescue to build the iso
# 4. use qemu to run the iso

require 'rake'

task :default => :run

# Target architecture
arch = ENV["ARCH"] || "x86_64"

# root directory for builds relative to the project root
build_root = ENV["BUILD_ROOT"] || "build"

# directory where all source files can be found relative to the project root
src_root = ENV["SRC_ROOT"] || "src"

# root for architecture-specific build files
arch_src_root = src_root.pathmap("%p/arch/#{arch}")
arch_build_root = build_root.pathmap("%p/arch/#{arch}")

# root of the generated kernel iso image
iso_root = build_root.pathmap("%p/isoroot")

kernel_name = "#{ENV["KERNEL_NAME"] || "kernel"}-#{arch}.bin"
kernel = "#{build_root}/#{kernel_name}"
target = "#{arch}-unknown-linux-gnu"
iso = "#{build_root}/rose-#{arch}.iso"
linker_script = "#{arch_src_root}/linker.ld"
grub_cfg_template = "#{arch_src_root}/grub.cfg.template"
grub_cfg = "#{iso_root}/boot/grub/grub.cfg"
asm_sources = Rake::FileList["#{arch_src_root}/*.asm"]
asm_objects = asm_sources.pathmap("%{^#{arch_src_root},#{arch_build_root}}X.o")
rust_pkg_name = `awk '/^name/{print $NF}' Cargo.toml`.gsub(/[\s"]/, "")
cargo_archive = "target/#{target}/debug/lib#{rust_pkg_name}.a"
rust_sources = Rake::FileList["src/**/*.rs"]
cpu = ENV["QEMU_CPU"] || "qemu64"

directory arch_build_root
directory "#{iso_root}/boot/grub"

desc "Generate kernel binary"
task :kernel => kernel

desc "Generate ISO image"
task :iso => iso

desc "Run cargo build"
task :cargo => cargo_archive

desc "Run rose using qemu"
task :run => iso do |t|
    qemu_flags = "-cpu #{cpu} -curses #{ENV["QEMU_FLAGS"]}"
    sh "qemu-system-x86_64 #{qemu_flags} -cdrom #{iso}"
end

desc "Clean up build files"
task :clean do |t|
    rm_rf build_root
    rm_rf cargo_archive.pathmap("%1d")
end

rule ".o" => [
        proc { |t| t.pathmap("%{^#{arch_build_root},#{arch_src_root}}X.asm")},
        arch_build_root
    ] do |t|
    sh "nasm -felf64 #{t.source} -o #{t.name}"
end

file cargo_archive => [*rust_sources, "Cargo.toml"] do |t|
    sh "cargo build --target #{target}"
end

file grub_cfg => [grub_cfg_template, "#{iso_root}/boot/grub"] do |t|
    cp grub_cfg_template, grub_cfg
    sh "sed -i s/KERNEL_BIN/#{kernel_name}/ #{grub_cfg}"
end

file kernel => [linker_script, *asm_objects, *cargo_archive] do |t|
    sh "ld -n --gc-sections -T #{linker_script} -o #{kernel} #{asm_objects} #{cargo_archive}"
end

file iso => [grub_cfg, kernel, "#{iso_root}/boot"] do |t|
    cp kernel, "#{iso_root}/boot/#{kernel_name}"
    sh "grub-mkrescue -o #{iso} #{iso_root}"
end

desc "Debug task which prints all paths relevant to the build"
task :paths do |t|
    puts "arch = #{arch}"
    puts "build_root = #{build_root}"
    puts "src_root = #{src_root}"
    puts "arch_src_root = #{arch_src_root}"
    puts "arch_build_root = #{arch_build_root}"
    puts "iso_root = #{iso_root}"
    puts "kernel_name = #{kernel_name}"
    puts "kernel = #{kernel}"
    puts "target = #{target}"
    puts "iso = #{iso}"
    puts "linker_script = #{linker_script}"
    puts "grub_cfg_template = #{grub_cfg_template}"
    puts "grub_cfg = #{grub_cfg}"
    puts "asm_sources = #{asm_sources}"
    puts "asm_objects = #{asm_objects}"
    puts "cargo_archive = #{cargo_archive}"
    puts "rust_pkg_name = #{rust_pkg_name}"
    puts "rust_sources = #{rust_sources}"
    puts "cpu = #{cpu}"
end
