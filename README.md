<div align="center">

# Marabunta
### A refreshingly simple ant colony rts game written in Rust

<br><br>
[![Play](https://gist.githubusercontent.com/cxmeel/0dbc95191f239b631c3874f4ccf114e2/raw/play.svg)](https://tvdboom.itch.io/marabunta)
<br><br>
</div>

<img src="https://github.com/tvdboom/marabunta/blob/master/assets/scenery/s1.png?raw=true" alt="Early game">
<img src="https://github.com/tvdboom/marabunta/blob/master/assets/scenery/s2.png?raw=true" alt="Traits">
<img src="https://github.com/tvdboom/marabunta/blob/master/assets/scenery/s3.png?raw=true" alt="Late game">
<img src="https://github.com/tvdboom/fortress/blob/master/assets/scenery/s4.png?raw=true" alt="Overview">

<br>

## 📜 Introduction

[Marabunta](https://en.wikipedia.org/wiki/Army_ant) is a real-time strategy game where you control an ant colony. Grow your colony,
expand your nest, gather resources, and fight for survival against scorpions, termites, wasps and
other ant colonies.

<br>

## 🎮 Gameplay

The goal of the game is to kill the queen of every enemy colony on the map. If your own queen
dies, you lose the game.

### Resources

There are two types of resources: leaves and nutrients. Resources are collected by worker ants
and brought to the queen. In turn, the queen consumes the resources to breed new ants. The
basic ant types require only leaves, stronger ant types like warriors or flying ants require
nutrients as well.

Leaves lie on the floor and can be encountered when digging new tunnels. Nutrients are
collected from corpses (be it ally or enemy). Corpses only lay for a limited time, so be
quick to collect the nutrients before they disappear.

### Enemies

The main enemies are other ant colonies. The game is won when all other queens are killed.
Besides ants, you can encounter scorpions when digging tunnels, and wasps and termites can
appear from holes on the ground.

### Commands

Every ant type behaves in a certain way and will go about their respective tasks (resource
collection, tunnel digging, enemy fighting, etc...) unless otherwise commanded by the player.
A player can select one or more ants and give them a command by right-clicking on the target
location or ant. The following commands are available:

- Click on a tunnel: Protect the location. The selected ants will wander close to the selected
  location. Use this, for example, to protect a tunnel entrance.
- Click on a leaf: Workers will harvest the leaf while other types will protect the location.
- Click on a corpse: Workers will harvest the corpse while other types will protect the location.
- Click on an unreachable location: Excavator ants will dig a tunnel to the target location.
  Other types will ignore this command.
- Click on an ally: The selected ants will protect the ally, meaning they will wander close to it.
- Click on an enemy: Attack the selected enemy.

Note that the queen cannot be given any commands unless the player has the `WanderinQueen` trait.
Use the `delete` key to remove all commands from the selected ants.

### Traits

Every fixed amount of time, the players can choose from one of three traits to improve their
colonies. There are many different traits with varying effects, ranging from improving the
capabilities of your ants, to reviving fallen ants or even having two queens.

<br>

### Key bindings

**Camera**
- `w-a-s-d`: Move the camera.
- `middle mouse button`: Move the camera.
- `scroll`: Zoom in/out.

**Ant control**
- `left-click`: Select ants (ctrl + click) to add to the selection.
- `right-click`: Move/dig/defend/attack target location/ant.
- `delete`: Remove player commands from selected ants.

**Others**
- `escape`: Enter/exit the menu.
- `space`: Pause/unpause the game.
- `m`: Toggle the audio settings.
