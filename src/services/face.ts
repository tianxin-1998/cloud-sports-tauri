import { invoke } from "./client";
import type { FaceAddResult, FaceGetResult, FaceListResult, FaceNextResult } from "../types";

export const faceAdd = (dataBase64: string) => invoke<FaceAddResult>("face_add", { dataBase64 });

export const faceList = () => invoke<FaceListResult>("face_list");

export const faceGet = (name: string) => invoke<FaceGetResult>("face_get", { name });

export const faceDelete = (name: string) => invoke<void>("face_delete", { name });

export const faceNext = () => invoke<FaceNextResult>("face_next");
