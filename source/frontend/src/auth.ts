
export type AuthProp = {
  signedIn: boolean,
  username: string | null
};

export const getAuthProp = (): AuthProp | null => {
  const v = localStorage.getItem("auth");

  if (v) {
    return JSON.parse(v);
  } else {
    return null;
  }
}

export const setAuthProp = (v: AuthProp | null) => {
  if (v) {
    localStorage.setItem("auth", JSON.stringify(v));
  } else {
    localStorage.removeItem("auth");
  }
}

