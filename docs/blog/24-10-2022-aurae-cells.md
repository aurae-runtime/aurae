# Isolation with Aurae Cells

Recently [Pull Request #73](https://github.com/aurae-runtime/aurae/pull/73) was merged and the Aurae project make the decision to pursue a new kind of isolation strategy with a concept we are calling `Cells`.

The concept of a cell is nothing new to any systemd or container connoisseur, as a cell is just a group of processes running in a unique cgroup namespace.