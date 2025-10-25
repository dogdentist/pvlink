
export type LinkType = {
  id: number,
  shortLink: string,
  longLink: string,
  created: number,
  expires: number,
  clicks: number
};

export type LinkClicks = {
  clicks: number
}

export type LinkCountryClicks = {
  country: string,
  clicks: number
}
