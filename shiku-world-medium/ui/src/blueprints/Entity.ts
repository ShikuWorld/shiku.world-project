// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.
import type { Body } from "./Body";
import type { Joint } from "./Joint";

export interface Entity { id: number, children: Array<Body>, joints: Record<number, Joint>, }