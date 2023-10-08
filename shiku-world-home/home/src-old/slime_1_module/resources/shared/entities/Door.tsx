<?xml version="1.0" encoding="UTF-8"?>
<tileset version="1.9" tiledversion="1.9.0" name="Door" tilewidth="130" tileheight="64" tilecount="11" columns="0" objectalignment="center">
 <grid orientation="orthogonal" width="1" height="1"/>
 <tile id="0">
  <properties>
   <property name="tile_name" value="default"/>
   <property name="variant" value="default"/>
  </properties>
  <image width="32" height="64" source="../images/bar_door.png"/>
  <objectgroup draworder="index" id="2">
   <object id="1" name="default" x="4" y="0" width="24" height="64"/>
  </objectgroup>
 </tile>
 <tile id="1">
  <properties>
   <property name="variant" value="golden"/>
  </properties>
  <image width="32" height="64" source="../images/bar_door_golden.png"/>
  <objectgroup draworder="index" id="2">
   <object id="1" name="default" x="4" y="0" width="24" height="64"/>
  </objectgroup>
 </tile>
 <tile id="2">
  <properties>
   <property name="variant" value="plattform_xl"/>
  </properties>
  <image width="130" height="30" source="../images/platform_large.png"/>
  <objectgroup draworder="index" id="2">
   <object id="1" name="default" x="8" y="7" width="113" height="17"/>
  </objectgroup>
 </tile>
 <tile id="3">
  <properties>
   <property name="variant" value="plattform_l"/>
  </properties>
  <image width="100" height="30" source="../images/platform_medium.png"/>
  <objectgroup draworder="index" id="2">
   <object id="1" name="default" x="10" y="7" width="79" height="17"/>
  </objectgroup>
 </tile>
 <tile id="4">
  <properties>
   <property name="variant" value="plattform_m"/>
  </properties>
  <image width="60" height="30" source="../images/platform_small.png"/>
  <objectgroup draworder="index" id="2">
   <object id="1" name="default" x="7" y="7" width="46" height="17"/>
  </objectgroup>
 </tile>
 <tile id="5">
  <properties>
   <property name="variant" value="plattform_s"/>
  </properties>
  <image width="40" height="20" source="../images/platform_smol.png"/>
  <objectgroup draworder="index" id="2">
   <object id="1" name="default" x="4" y="2" width="31" height="17"/>
  </objectgroup>
 </tile>
 <tile id="6">
  <properties>
   <property name="variant" value="forest_s"/>
  </properties>
  <image width="16" height="8" source="../images/forest_plattform_1.png"/>
  <objectgroup draworder="index" id="2">
   <object id="2" name="default" x="0" y="0" width="16" height="6"/>
  </objectgroup>
 </tile>
 <tile id="7">
  <properties>
   <property name="variant" value="forest_m"/>
  </properties>
  <image width="32" height="8" source="../images/forest_plattform_2.png"/>
  <objectgroup draworder="index" id="2">
   <object id="1" name="default" x="0" y="0" width="32" height="6"/>
  </objectgroup>
 </tile>
 <tile id="8">
  <properties>
   <property name="variant" value="forest_l"/>
  </properties>
  <image width="48" height="9" source="../images/forest_plattform_3.png"/>
  <objectgroup draworder="index" id="2">
   <object id="1" name="default" x="0" y="0" width="48" height="6"/>
  </objectgroup>
 </tile>
 <tile id="9">
  <properties>
   <property name="variant" value="tree_left"/>
  </properties>
  <image width="64" height="16" source="../images/forest_plattform_4.png"/>
  <objectgroup draworder="index" id="2">
   <object id="1" name="default" x="0" y="0" width="32" height="6"/>
  </objectgroup>
 </tile>
 <tile id="10">
  <properties>
   <property name="variant" value="tree_right"/>
  </properties>
  <image width="64" height="16" source="../images/forest_plattform_5.png"/>
  <objectgroup draworder="index" id="2">
   <object id="1" name="default" x="32" y="0" width="32" height="6"/>
  </objectgroup>
 </tile>
</tileset>
