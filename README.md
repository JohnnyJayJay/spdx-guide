# spdx-guide
`spdx-guide` is a super simple command line tool to get you started with the creation of **simple** [SPDX](https://github.com/david-a-wheeler/spdx-tutorial) files.

## What is SPDX?
SPDX is a standard for specifying software licenses and creating Software Bills of Materials (SBOMs) 
that give licensing information for software and its dependencies in a machine-readable format. 
Adding an SPDX file to your projects, especially to your libraries, helps others and contributes to the 
greater goal of fixing the mess that currently is open source licensing.

## Goals of spdx-guide
This software was specifically created to get people who have never used or heard of SPDX to create SPDX files for their projects.
spdx-guide will *guide* you through the creation process in an intuitive command line dialogue.

There is existing tooling for working with the SPDX framework, but from my experience none of it is aimed at 
complete beginners or people who want to improve their licensing situation but aren't ready to invest a lot of their 
time into memorising specifications or configuring their build tools.

In short, the goals of this software are to
- enable people who do not know anything about SPDX to create SPDX files for their projects
- provide more advanced users with a tool that creates a simple SPDX file that can later be extended with more sophisticated tools

## Non-goals
spdx-guide *will not* (for now):
- process the dependencies of your project
- perform license detection or analysis
- create a complete SBOM for your distributions 
- give you legal advice
- pick a license for you

## License
spdx-guide is free software and licensed under the GNU General Public License version 3.0 or any later version, at your discretion.
