{ config, pkgs, ... }:

let
  folderToUpload = builtins.path {
    path = ./init-volume.d;
    name = "volumes";
  };
in
{
  system.activationScripts.init-c-volumes = {
    deps = [ "specialfs" ];
    text = ''
      echo '[init container volumes (mod)] Setting up'
      target="/var/lib/podman/volumes"
      mkdir -p "$target"

      # Iterate through folders in folderToUpload
      for source_folder in ${folderToUpload}/*; do
        if [ -d "$source_folder" ]; then
          folder_name=$(basename "$source_folder")
          target_folder="$target/$folder_name"

          # Check if target folder exists and is empty
          if [ ! -d "$target_folder" ] || [ -z "$(ls -A "$target_folder")" ]; then
            echo "Copying $folder_name to $target_folder"
            mkdir -p "$target_folder"
            cp -r "$source_folder"/* "$target_folder"
          else
            echo "Skipping $folder_name, target folder exists and is not empty"
          fi
        fi
      done

      # Iterate through folders in $target and unzip matching zip files
      for folder in "$target"/*; do
        if [ -d "$folder" ]; then
          folder_name=$(basename "$folder")
          zip_file="$folder/$folder_name.zip"
          if [ -f "$zip_file" ]; then
            echo "Unzipping $zip_file into $folder"
            ${pkgs.unzip}/bin/unzip -o "$zip_file" -d "$folder" >/dev/null2>&1
            rm $zip_file
          fi
        fi
      done
    '';
  };
}
