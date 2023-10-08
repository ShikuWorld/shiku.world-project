<?xml version="1.0" encoding="UTF-8"?>
<tileset version="1.9" tiledversion="1.9.0" name="WallPlatformOpener" tilewidth="64" tileheight="40" tilecount="49" columns="7" objectalignment="center">
 <image source="../images/Pressure_P_glow_forest.png" width="448" height="280"/>
 <tile id="0">
  <properties>
   <property name="tile_name" value="default"/>
   <property name="variant" value="default"/>
  </properties>
  <objectgroup draworder="index" id="2">
   <object id="1" name="default" x="25" y="14" width="24" height="14"/>
  </objectgroup>
 </tile>
 <tile id="1">
  <properties>
   <property name="tile_name" value="off"/>
  </properties>
 </tile>
 <tile id="2">
  <properties>
   <property name="tile_name" value="on"/>
  </properties>
  <animation>
   <frame tileid="2" duration="130"/>
   <frame tileid="3" duration="130"/>
   <frame tileid="4" duration="130"/>
   <frame tileid="3" duration="130"/>
  </animation>
 </tile>
 <tile id="7">
  <properties>
   <property name="variant" value="yellow"/>
  </properties>
  <objectgroup draworder="index" id="2">
   <object id="1" name="default" x="26.5455" y="15.5455" width="24" height="14"/>
  </objectgroup>
 </tile>
 <tile id="9">
  <animation>
   <frame tileid="9" duration="130"/>
   <frame tileid="10" duration="130"/>
   <frame tileid="11" duration="130"/>
   <frame tileid="10" duration="130"/>
  </animation>
 </tile>
 <tile id="14">
  <properties>
   <property name="variant" value="blue"/>
  </properties>
  <objectgroup draworder="index" id="2">
   <object id="1" name="default" x="26.5455" y="15.5455" width="24" height="14"/>
  </objectgroup>
 </tile>
 <tile id="16">
  <animation>
   <frame tileid="16" duration="130"/>
   <frame tileid="17" duration="130"/>
   <frame tileid="18" duration="130"/>
   <frame tileid="17" duration="130"/>
  </animation>
 </tile>
 <tile id="22">
  <animation>
   <frame tileid="22" duration="100"/>
   <frame tileid="23" duration="100"/>
   <frame tileid="24" duration="100"/>
   <frame tileid="23" duration="100"/>
  </animation>
 </tile>
 <tile id="29">
  <animation>
   <frame tileid="29" duration="100"/>
   <frame tileid="30" duration="100"/>
   <frame tileid="31" duration="100"/>
   <frame tileid="30" duration="100"/>
  </animation>
 </tile>
 <tile id="35">
  <properties>
   <property name="variant" value="no_heart"/>
  </properties>
 </tile>
 <tile id="37">
  <animation>
   <frame tileid="37" duration="130"/>
   <frame tileid="38" duration="130"/>
   <frame tileid="39" duration="130"/>
   <frame tileid="38" duration="130"/>
  </animation>
 </tile>
 <tile id="42">
  <objectgroup draworder="index" id="2">
   <object id="1" name="default" x="27" y="15.5455" width="18" height="13"/>
  </objectgroup>
 </tile>
</tileset>
