<?xml version="1.0" encoding="UTF-8"?>
<tileset version="1.9" tiledversion="1.9.0" name="WallPlatform" tilewidth="16" tileheight="16" tilecount="8" columns="2" objectalignment="center">
 <image source="../images/forest-walls.png" width="32" height="64"/>
 <tile id="0">
  <properties>
   <property name="tile_name" value="on"/>
   <property name="variant" value="default"/>
  </properties>
  <objectgroup draworder="index" id="2">
   <object id="1" name="platform" x="0" y="0" width="16" height="16"/>
   <object id="2" name="sensor" x="1" y="1" width="14" height="14"/>
  </objectgroup>
 </tile>
 <tile id="1">
  <properties>
   <property name="tile_name" value="off"/>
  </properties>
 </tile>
 <tile id="2">
  <properties>
   <property name="variant" value="red"/>
  </properties>
  <objectgroup draworder="index" id="2">
   <object id="1" name="platform" x="0" y="0" width="16" height="16"/>
   <object id="2" name="sensor" x="1" y="1" width="14" height="14"/>
  </objectgroup>
 </tile>
 <tile id="4">
  <properties>
   <property name="variant" value="blue"/>
  </properties>
  <objectgroup draworder="index" id="2">
   <object id="1" name="platform" x="0" y="0" width="16" height="16"/>
   <object id="2" name="sensor" x="1" y="1" width="14" height="14"/>
  </objectgroup>
 </tile>
 <tile id="6">
  <properties>
   <property name="variant" value="yellow"/>
  </properties>
  <objectgroup draworder="index" id="2">
   <object id="1" name="platform" x="0" y="0" width="16" height="16"/>
   <object id="2" name="sensor" x="1" y="1" width="14" height="14"/>
  </objectgroup>
 </tile>
</tileset>
