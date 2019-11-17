$ScriptPath = (Get-Item -Path ".\" -Verbose).FullName + "\"
$BuildFolder = ($ScriptPath + "build\")
New-Item -ItemType Directory -Force -Path $BuildFolder
Copy-Item -Path ($ScriptPath + "target\release\*") -Destination ($BuildFolder) -Force
Copy-Item -Path ($ScriptPath + "resources\") -Recurse -Destination $BuildFolder -Container -Force