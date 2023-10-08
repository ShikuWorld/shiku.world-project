<?xml version="1.0" encoding="UTF-8"?>
<tileset version="1.9" tiledversion="1.9.0" name="OpeningAreaPlate" tilewidth="64" tileheight="40" tilecount="49" columns="7" objectalignment="center">
 <image source="../images/opening_area.png" width="448" height="280"/>
 <tile id="1">
  <animation>
   <frame tileid="2" duration="100"/>
   <frame tileid="3" duration="100"/>
   <frame tileid="4" duration="100"/>
   <frame tileid="5" duration="100"/>
  </animation>
 </tile>
 <tile id="7">
  <properties>
   <property name="tile_name" value="default"/>
   <property name="variant" value="default"/>
  </properties>
  <objectgroup draworder="index" id="2">
   <object id="1" name="platform" x="24" y="28" width="24" height="6"/>
   <object id="2" name="activation" x="29" y="16" width="14" height="12"/>
  </objectgroup>
 </tile>
 <tile id="8">
  <properties>
   <property name="tile_name" value="activated"/>
  </properties>
  <animation>
   <frame tileid="8" duration="100"/>
   <frame tileid="9" duration="100"/>
   <frame tileid="10" duration="100"/>
   <frame tileid="9" duration="100"/>
  </animation>
 </tile>
 <tile id="13">
  <properties>
   <property name="tile_name" value="done"/>
  </properties>
 </tile>
 <tile id="14">
  <properties>
   <property name="variant" value="portal"/>
  </properties>
  <objectgroup draworder="index" id="4">
   <object id="13" name="platform" x="24" y="28" width="24" height="6"/>
   <object id="14" name="activation" x="29" y="16" width="14" height="12"/>
  </objectgroup>
 </tile>
 <tile id="15">
  <animation>
   <frame tileid="15" duration="100"/>
   <frame tileid="16" duration="100"/>
   <frame tileid="17" duration="100"/>
   <frame tileid="16" duration="100"/>
  </animation>
 </tile>
 <tile id="21">
  <properties>
   <property name="variant" value="portal_no_glow"/>
  </properties>
  <objectgroup draworder="index" id="3">
   <object id="3" name="platform" x="23.75" y="28.125" width="24" height="6"/>
   <object id="4" name="activation" x="29" y="16" width="14" height="12"/>
  </objectgroup>
 </tile>
 <tile id="22">
  <animation>
   <frame tileid="23" duration="100"/>
   <frame tileid="24" duration="100"/>
   <frame tileid="25" duration="100"/>
   <frame tileid="26" duration="100"/>
  </animation>
 </tile>
 <tile id="28">
  <properties>
   <property name="variant" value="portal_green"/>
  </properties>
  <objectgroup draworder="index" id="2">
   <object id="1" name="platform" x="24" y="28" width="24" height="6"/>
   <object id="2" name="activation" x="29" y="16" width="14" height="12"/>
  </objectgroup>
 </tile>
 <tile id="29">
  <animation>
   <frame tileid="30" duration="100"/>
   <frame tileid="31" duration="100"/>
   <frame tileid="32" duration="100"/>
   <frame tileid="33" duration="100"/>
  </animation>
 </tile>
 <tile id="35">
  <properties>
   <property name="variant" value="portal_green_no_glow"/>
  </properties>
  <objectgroup draworder="index" id="2">
   <object id="1" name="platform" x="24" y="28" width="24" height="6"/>
   <object id="2" name="activation" x="29" y="16" width="14" height="12"/>
  </objectgroup>
 </tile>
 <tile id="36">
  <animation>
   <frame tileid="40" duration="100"/>
   <frame tileid="39" duration="100"/>
   <frame tileid="38" duration="100"/>
   <frame tileid="39" duration="100"/>
  </animation>
 </tile>
 <tile id="42">
  <properties>
   <property name="variant" value="green_opener"/>
  </properties>
  <objectgroup draworder="index" id="2">
   <object id="1" name="platform" x="24" y="28" width="24" height="6"/>
   <object id="2" name="activation" x="29" y="16" width="14" height="12"/>
  </objectgroup>
 </tile>
 <tile id="43">
  <animation>
   <frame tileid="43" duration="100"/>
   <frame tileid="44" duration="100"/>
   <frame tileid="45" duration="100"/>
   <frame tileid="44" duration="100"/>
  </animation>
 </tile>
</tileset>
