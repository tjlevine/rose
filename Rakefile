# things this rakefile should do:
# 1. use nasm to compile .asm files to .o files
# 2. use ld to link the .o files
# 3. use grub-mkrescue to build the iso
# 4. use qemu to run the iso

require 'rake'

task :default => :test

# Target architecture
arch = ENV["ARCH"] || "x86_64"

# root directory for builds relative to the project root
build_root = ENV["BUILD_ROOT"] || "build"

# directory where all source files can be found relative to the project root
src_root = ENV["SRC_ROOT"] || "src"

# root for archetecture-specific build files
arch_src_root = src_root.pathmap("%p/arch/#{arch}")
arch_build_root = build_root.pathmap("%p/arch/#{arch}")

# root of the generated kernel iso image
iso_root = build_root.pathmap("%p/iso")

kernel_name = "#{ENV["KERNEL_NAME"] || "kernel"}-#{arch}.bin"
kernel = "#{build_root}/#{kernel_name}"
iso = "#{build_root}/rose-#{arch}.iso"
linker_script = "#{arch_src_root}/linker.ld"
grub_cfg_template = "#{arch_src_root}/grub.cfg.template"
asm_sources = Rake::FileList["#{arch_src_root}/*.asm"]
asm_objects = asm_sources.pathmap("%{^#{arch_src_root},#{arch_build_root}}X.o")

directory "#{arch_build_root}"
directory "#{iso_root}/boot/grub"

task :asm_objects => [arch_build_root] + asm_objects
task :kernel => kernel
task :iso => iso
task :iso_files => [:iso_grub_cfg, :iso_kernel]
task :iso_kernel => "#{iso_root}/boot/#{kernel_name}"
task :iso_grub_cfg => "#{iso_root}/boot/grub/grub.cfg"
task :run => iso do |t|
    sh "/usr/bin/qemu-system-x86_64 -curses -cdrom #{iso}"
end
task :clean do |t|
    rm_rf build_root
end
task :test do |t|
    puts "arch: #{arch}"
    puts "build_root: #{build_root}"
    puts "src_root: #{src_root}"
    puts "iso_root: #{iso_root}"
    puts "kernel: #{kernel}"
    puts "kernel_name: #{kernel_name}"
    puts "iso: #{iso}"
    puts "arch_build_root: #{arch_build_root}"
    puts "arch_src_root: #{arch_src_root}"
    puts "linker script: #{linker_script}"
    puts "grub_cfg_template: #{grub_cfg_template}"
    puts "asm_sources: #{asm_sources}"
    puts "asm_objects: #{asm_objects}"
end

rule ".o" => proc { |task_name| task_name.pathmap("%{^#{arch_build_root},#{arch_src_root}}X.asm")} do |t|
    sh "nasm -felf64 #{t.source} -o #{t.name}"
end

file "#{iso_root}/boot/grub/grub.cfg" => [grub_cfg_template, "#{iso_root}/boot/grub"] do |t|
    cp grub_cfg_template, "#{iso_root}/boot/grub/grub.cfg"
    sh "sed -i s/KERNEL_BIN/#{kernel_name}/ #{iso_root}/boot/grub/grub.cfg"
end

file "#{iso_root}/boot/#{kernel_name}" => [kernel, "#{iso_root}/boot"] do |t|
    cp kernel, "#{iso_root}/boot/#{kernel_name}"
end

file kernel => [linker_script, :asm_objects] do |t|
    sh "ld -n -T #{linker_script} -o #{kernel} #{asm_objects}"
end

file iso => :iso_files do |t|
    sh "grub-mkrescue -o #{iso} #{iso_root}"
end

