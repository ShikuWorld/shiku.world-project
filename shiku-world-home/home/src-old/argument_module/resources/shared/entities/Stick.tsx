<?xml version="1.0" encoding="UTF-8"?>
<tileset version="1.9" tiledversion="1.9.2" name="Stick" tilewidth="60" tileheight="60" tilecount="10" columns="5" objectalignment="center">
 <image source="../images/chars/stick.png" width="300" height="120"/>
 <tile id="0">
  <properties>
   <property name="tile_name" value="default"/>
   <property name="variant" value="default"/>
  </properties>
  <objectgroup draworder="index" id="2">
   <object id="1" name="default" x="8" y="4" width="50" height="50"/>
  </objectgroup>
  <animation>
   <frame tileid="0" duration="33"/>
   <frame tileid="1" duration="33"/>
   <frame tileid="2" duration="50"/>
   <frame tileid="3" duration="66"/>
   <frame tileid="4" duration="66"/>
   <frame tileid="5" duration="50"/>
   <frame tileid="6" duration="33"/>
   <frame tileid="7" duration="33"/>
   <frame tileid="8" duration="33"/>
   <frame tileid="9" duration="33"/>
  </animation>
 </tile>
 <tile id="1">
  <properties>
   <property name="tile_name" value="backwards"/>
  </properties>
  <animation>
   <frame tileid="9" duration="33"/>
   <frame tileid="8" duration="33"/>
   <frame tileid="7" duration="33"/>
   <frame tileid="6" duration="33"/>
   <frame tileid="5" duration="50"/>
   <frame tileid="4" duration="66"/>
   <frame tileid="3" duration="66"/>
   <frame tileid="2" duration="50"/>
   <frame tileid="1" duration="33"/>
   <frame tileid="0" duration="33"/>
  </animation>
 </tile>
</tileset>
