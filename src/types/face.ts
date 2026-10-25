export interface FacePhoto {
  name: string;
}

export interface FaceListResult {
  photos: FacePhoto[];
}

export interface FaceGetResult {
  data_base64: string;
}

export interface FaceNextResult {
  data_base64: string;
  name: string;
}

export interface FaceAddResult {
  name: string;
}
