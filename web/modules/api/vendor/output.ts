import type { APIError, Reference, ReferenceTypes, ReferenceTypesObject, Email, Username, LoginSpecifier, Password, LoginOptions, CSRF, PasswordObject, SessionState, RegisterOptions, EmailObject, UpdateEmail, UpdatePassword, UsernameObject, AccountSettings, AccountSettingsPatch, Notification, NotificationList, Base64, UserRole, UserIdentity, HTML, UserInfo, UserProfile, UserClientGetResponse, Wikitext, UserProfilePatch, AvatarURL, AvatarURLObject, UserBlockedList, UserGetResponse, UserBlocked, SiteName, Membership, MembershipList, ApplicationSendList, Invite, InviteList, MembershipStatus, ApplicationSend, MembershipRole, InviteSend, Slug, PageCreateOptions, TagList, Page, WikitextObject, HTMLObject, FTMLSyntaxTree, PageGetResponse, PagePatch, SlugObject, Paginated, Revision, RevisionHistory, RevisionGetResponse, RevisionPatch, TagListObject, Score, CastVotePlus, CastVotePlusMinus, CastVoteStar, Vote, VoterList, CastVote, CastVoteStatus, CastVoteObject, Mime, FileMetadata, FileMetadataList, BinaryData, FileUpload, FileSiteMetadata, Report, ReportList, ReportSend, Message, MessageList, MessagePatch, MessageSend, ForumCreationContext, ForumCategory, ForumGroup, Forum, ForumGroupList, ForumGroupPatch, ForumCategoryCreate, ForumCategoryList, ForumCategoryPatch, ForumThreadCreate, ForumSortingTypes, ForumThread, ForumThreadList, ForumThreadPatch, ForumPostCreate, ForumPost, ForumPostList, ForumPostPatch, UserKick, UserBanned, UserBannedList, UserBan, Category, CategoryList, CategoryDefault, CategoryDefaultPatch, CategoryPatch, SiteSettings, SiteSettingsPatch, Application, ApplicationList, CreateSiteSettings, SiteNewsletter, SiteTransfer } from "./types";
export const defaults: RequestOptions = {
    baseUrl: "https://wikijump.com/api--v1"
};
export const servers = {
    contextlessPublicApi: "https://wikijump.com/api--v1",
    siteContextPublicApi: ({ site = "www" }: {
        site: string | number | boolean;
    }) => `https://${site}.wikijump.com/api--v1`
};
export type RequestOptions = {
    baseUrl?: string;
    fetch?: typeof fetch;
    headers?: Record<string, string | undefined>;
} & Omit<RequestInit, "body" | "headers">;
export type ApiResponse<T> = {
    status: number;
    statusText: string;
    headers: Record<string, string>;
    data: T;
};
type Encoders = Array<(s: string) => string>;
type TagFunction = (strings: TemplateStringsArray, ...values: any[]) => string;
type FetchRequestOptions = RequestOptions & {
    body?: string | FormData;
};
type JsonRequestOptions = RequestOptions & {
    body: unknown;
};
type FormRequestOptions<T extends Record<string, unknown>> = RequestOptions & {
    body: T;
};
type MultipartRequestOptions = RequestOptions & {
    body: Record<string, any>; // string | Blob
};
/** Utilities functions */
export const _ = {
    unwrap<T>(res: Promise<ApiResponse<T>>): Promise<T> {
      return res.then(r => {
        if (r.data !== undefined) return r.data
      }) as Promise<T>
    },
    // Encode param names and values as URIComponent
    encodeReserved: [encodeURI, encodeURIComponent],
    allowReserved: [encodeURI, encodeURI],
    /** Deeply remove all properties with undefined values. */
    stripUndefined<T extends Record<string, U | undefined>, U>(obj?: T): Record<string, U> | undefined {
        return obj && JSON.parse(JSON.stringify(obj));
    },
    isEmpty(v: unknown): boolean {
        return typeof v === "object" && !!v ?
            Object.keys(v).length === 0 && v.constructor === Object :
            v === undefined;
    },
    /** Creates a tag-function to encode template strings with the given encoders. */
    encode(encoders: Encoders, delimiter = ","): TagFunction {
        return (strings: TemplateStringsArray, ...values: any[]) => {
            return strings.reduce((prev, s, i) => `${prev}${s}${q(values[i] ?? "", i)}`, "");
        };
        function q(v: any, i: number): string {
            const encoder = encoders[i % encoders.length];
            if (typeof v === "object") {
                if (Array.isArray(v)) {
                    return v.map(encoder).join(delimiter);
                }
                const flat = Object.entries(v).reduce((flat, entry) => [...flat, ...entry], [] as any);
                return flat.map(encoder).join(delimiter);
            }
            return encoder(String(v));
        }
    },
    /** Separate array values by the given delimiter. */
    delimited(delimiter = ","): (params: Record<string, any>, encoders?: Encoders) => string {
        return (params: Record<string, any>, encoders = _.encodeReserved) => Object.entries(params)
            .filter(([, value]) => !_.isEmpty(value))
            .map(([name, value]) => _.encode(encoders, delimiter) `${name}=${value}`)
            .join("&");
    },
    /** Join URLs parts. */
    joinUrl(...parts: Array<string | undefined>): string {
        return parts
            .filter(Boolean)
            .join("/")
            .replace(/([^:]\/)\/+/, "$1");
    }
};
/** Functions to serialize query parameters in different styles. */
export const QS = {
    /** Join params using an ampersand and prepends a questionmark if not empty. */
    query(...params: string[]): string {
        const s = params.filter(p => !!p).join("&");
        return s && `?${s}`;
    },
    /**
     * Serializes nested objects according to the `deepObject` style specified in
     * https://github.com/OAI/OpenAPI-Specification/blob/master/versions/3.0.0.md#style-values
     */
    deep(params: Record<string, any>, [k, v] = _.encodeReserved): string {
        const qk = _.encode([(s) => s, k]);
        const qv = _.encode([(s) => s, v]);
        // don't add index to arrays
        // https://github.com/expressjs/body-parser/issues/289
        const visit = (obj: any, prefix = ""): string => Object.entries(obj)
            .filter(([, v]) => !_.isEmpty(v))
            .map(([prop, v]) => {
            const isValueObject = typeof v === "object";
            const index = Array.isArray(obj) && !isValueObject ? "" : prop;
            const key = prefix ? qk `${prefix}[${index}]` : prop;
            if (isValueObject) {
                return visit(v, key);
            }
            return qv `${key}=${v}`;
        })
            .join("&");
        return visit(params);
    },
    /**
     * Property values of type array or object generate separate parameters
     * for each value of the array, or key-value-pair of the map.
     * For other types of properties this property has no effect.
     * See https://github.com/OAI/OpenAPI-Specification/blob/master/versions/3.0.0.md#encoding-object
     */
    explode(params: Record<string, any>, encoders = _.encodeReserved): string {
        const q = _.encode(encoders);
        return Object.entries(params)
            .filter(([, value]) => typeof value !== "undefined")
            .map(([name, value]) => {
            if (Array.isArray(value)) {
                return value.map((v) => q `${name}=${v}`).join("&");
            }
            if (typeof value === "object") {
                return QS.explode(value, encoders);
            }
            return q `${name}=${value}`;
        })
            .join("&");
    },
    form: _.delimited(),
    pipe: _.delimited("|"),
    space: _.delimited("%20"),
};
/** Http request base methods. */
export const http = {
    async fetch(url: string, req?: FetchRequestOptions): Promise<ApiResponse<any>> {
        const { baseUrl, headers, fetch: customFetch, ...init } = { ...defaults, ...req };
        const href = _.joinUrl(baseUrl, url);
        const res = await (customFetch || fetch)(href, {
            ...init,
            headers: _.stripUndefined({ ...defaults.headers, ...headers }),
        });
        let text: string | undefined;
        try {
            text = await res.text();
        }
        catch (err) { /* ok */ }
        if (!res.ok) {
            throw new HttpError(res.status, res.statusText, href, res.headers, text);
        }
        return {
            status: res.status,
            statusText: res.statusText,
            headers: http.headers(res.headers),
            data: text
        };
    },
    async fetchJson(url: string, req: FetchRequestOptions = {}): Promise<ApiResponse<any>> {
        const res = await http.fetch(url, {
            ...req,
            headers: {
                ...req.headers,
                Accept: "application/json",
            },
        });
        res.data = res.data && JSON.parse(res.data);
        return res;
    },
    async fetchVoid(url: string, req: FetchRequestOptions = {}): Promise<ApiResponse<undefined>> {
        const res = await http.fetch(url, {
            ...req,
            headers: {
                ...req.headers,
                Accept: "application/json",
            },
        });
        return res as ApiResponse<undefined>;
    },
    json({ body, headers, ...req }: JsonRequestOptions): FetchRequestOptions {
        return {
            ...req,
            body: JSON.stringify(body),
            headers: {
                ...headers,
                "Content-Type": "application/json",
            },
        };
    },
    form<T extends Record<string, unknown>>({ body, headers, ...req }: FormRequestOptions<T>): FetchRequestOptions {
        return {
            ...req,
            body: QS.form(body),
            headers: {
                ...headers,
                "Content-Type": "application/x-www-form-urlencoded",
            },
        };
    },
    multipart({ body, ...req }: MultipartRequestOptions): FetchRequestOptions {
        const data = new FormData();
        Object.entries(body).forEach(([name, value]) => {
            data.append(name, value);
        });
        return {
            ...req,
            body: data,
        };
    },
    headers(headers: Headers): Record<string, string> {
        const res: Record<string, string> = {};
        headers.forEach((value, key) => res[key] = value);
        return res;
    }
};
export class HttpError extends Error {
    status: number;
    statusText: string;
    headers: Record<string, string>;
    data?: Record<string, unknown>;
    constructor(status: number, statusText: string, url: string, headers: Headers, text?: string) {
        super(`${url} - ${statusText} (${status})`);
        this.status = status;
        this.statusText = statusText;
        this.headers = http.headers(headers);
        if (text) {
            try {
                this.data = JSON.parse(text);
            }
            catch (err) { /* ok */ }
        }
    }
}
/** Utility Type to extract returns type from a method. */
export type ApiResult<Fn> = Fn extends (...args: any) => Promise<ApiResponse<infer T>> ? T : never;
export class API {
/**
 * INCOMPLETE - STUB
 */
async queryRequest(options?: RequestOptions): Promise<void> {
    return await _.unwrap(http.fetchVoid("/query", {
        ...options,
        method: "POST"
    }));
}
/**
 * Resolves an ID and returns what type of object it refers to.
 */
async utilResolveId(id: Reference, options?: RequestOptions): Promise<ReferenceTypesObject> {
    return await _.unwrap(http.fetchJson(`/util/resolveid/${id}`, {
        ...options
    }));
}
/**
 * Attempts a login. The login specifier can be either a username or an email address.
 */
async authLogin(loginOptions: LoginOptions, options?: RequestOptions): Promise<CSRF> {
    return await _.unwrap(http.fetchJson("/auth/login", http.json({
        ...options,
        method: "POST",
        body: loginOptions
    })));
}
/**
 * Confirms the client's password.
 */
async authConfirm(passwordObject: PasswordObject, options?: RequestOptions): Promise<void> {
    return await _.unwrap(http.fetchVoid("/auth/confirm", http.json({
        ...options,
        method: "POST",
        body: passwordObject
    })));
}
/**
 * Logs the client out.
 */
async authLogout(options?: RequestOptions): Promise<void> {
    return await _.unwrap(http.fetchVoid("/auth/logout", {
        ...options,
        method: "DELETE"
    }));
}
/**
 * Gets the authentication state of the client.
 */
async authCheck(options?: RequestOptions): Promise<SessionState> {
    return await _.unwrap(http.fetchJson("/auth/check", {
        ...options,
        method: "POST"
    }));
}
/**
 * Refreshes the client's session.
 */
async authRefresh(options?: RequestOptions): Promise<CSRF> {
    return await _.unwrap(http.fetchJson("/auth/refresh", {
        ...options,
        method: "POST"
    }));
}
/**
 * Registers an account. Email validation will be required.
 */
async accountRegister(registerOptions: RegisterOptions, options?: RequestOptions): Promise<CSRF> {
    return await _.unwrap(http.fetchJson("/account/register", http.json({
        ...options,
        method: "POST",
        body: registerOptions
    })));
}
/**
 * Sends a verification email to the account's address.
 */
async accountSendVerificationEmail(options?: RequestOptions): Promise<void> {
    return await _.unwrap(http.fetchVoid("/account/send-verification-email", {
        ...options,
        method: "POST"
    }));
}
/**
 * Starts the deletion process for an account. Requires additional email validation for the process to complete.
 *
 */
async accountRequestDeletion(options?: RequestOptions): Promise<void> {
    return await _.unwrap(http.fetchVoid("/account/request-deletion", {
        ...options,
        method: "POST"
    }));
}
/**
 * Starts the password recovery routine.
 */
async accountStartRecovery(emailObject: EmailObject, options?: RequestOptions): Promise<void> {
    return await _.unwrap(http.fetchVoid("/account/start-recovery", http.json({
        ...options,
        method: "POST",
        body: emailObject
    })));
}
/**
 * Gets the current email address.
 */
async accountGetEmail(options?: RequestOptions): Promise<EmailObject> {
    return await _.unwrap(http.fetchJson("/account/email", {
        ...options
    }));
}
/**
 * Updates the current email address. Does not immediately change the email, as the change must be verified through a link that is sent to the requested email.
 *
 */
async accountUpdateEmail(updateEmail: UpdateEmail, options?: RequestOptions): Promise<void> {
    return await _.unwrap(http.fetchVoid("/account/email", http.json({
        ...options,
        method: "PUT",
        body: updateEmail
    })));
}
/**
 * Updates the current password.
 */
async accountUpdatePassword(updatePassword: UpdatePassword, options?: RequestOptions): Promise<void> {
    return await _.unwrap(http.fetchVoid("/account/password", http.json({
        ...options,
        method: "PUT",
        body: updatePassword
    })));
}
/**
 * Gets the current username.
 */
async accountGetUsername(options?: RequestOptions): Promise<UsernameObject> {
    return await _.unwrap(http.fetchJson("/account/username", {
        ...options
    }));
}
/**
 * Updates the current username.
 */
async accountUpdateUsername(usernameObject: UsernameObject, options?: RequestOptions): Promise<void> {
    return await _.unwrap(http.fetchVoid("/account/username", http.json({
        ...options,
        method: "PUT",
        body: usernameObject
    })));
}
/**
 * Gets the current account settings.
 */
async accountGetSettings(options?: RequestOptions): Promise<AccountSettings> {
    return await _.unwrap(http.fetchJson("/account/settings", {
        ...options
    }));
}
/**
 * Update (patch) the client's user details.
 */
async accountUpdateSettings(accountSettingsPatch: AccountSettingsPatch, options?: RequestOptions): Promise<void> {
    return await _.unwrap(http.fetchVoid("/account/settings", http.json({
        ...options,
        method: "PATCH",
        body: accountSettingsPatch
    })));
}
/**
 * Gets the client's current notifications.
 */
async notificationGet(options?: RequestOptions): Promise<NotificationList> {
    return await _.unwrap(http.fetchJson("/notification", {
        ...options
    }));
}
/**
 * Dismisses all of the client's notifications.
 */
async notificationDismissAll(options?: RequestOptions): Promise<void> {
    return await _.unwrap(http.fetchVoid("/notification", {
        ...options,
        method: "DELETE"
    }));
}
/**
 * Gets the client's user details.
 */
async userClientGet({ detail, avatars }: {
    detail?: "identity" | "info" | "profile";
    avatars?: boolean;
} = {}, options?: RequestOptions): Promise<UserClientGetResponse> {
    return await _.unwrap(http.fetchJson(`/user${QS.query(QS.form({
        detail,
        avatars
    }))}`, {
        ...options
    }));
}
/**
 * Update (patch) the client's user details.
 */
async userClientUpdateProfile(userProfilePatch: UserProfilePatch, options?: RequestOptions): Promise<void> {
    return await _.unwrap(http.fetchVoid("/user", http.json({
        ...options,
        method: "PATCH",
        body: userProfilePatch
    })));
}
/**
 * Gets the client's avatar. This won't return the avatar directly, but rather return the URL for it.
 */
async userClientGetAvatar(options?: RequestOptions): Promise<AvatarURLObject> {
    return await _.unwrap(http.fetchJson("/user/avatar", {
        ...options
    }));
}
/**
 * Sets the client's avatar.
 */
async userClientSetAvatar(body: string, options?: RequestOptions): Promise<void> {
    return await _.unwrap(http.fetchVoid("/user/avatar", {
        ...options,
        method: "POST",
        body
    }));
}
/**
 * Removes the client's avatar.
 */
async userClientRemoveAvatar(options?: RequestOptions): Promise<void> {
    return await _.unwrap(http.fetchVoid("/user/avatar", {
        ...options,
        method: "DELETE"
    }));
}
/**
 * Gets the list of users the client has blocked.
 */
async userClientGetBlocked(options?: RequestOptions): Promise<UserBlockedList> {
    return await _.unwrap(http.fetchJson("/user/blocked", {
        ...options
    }));
}
/**
 * Gets a user's details.
 */
async userGet(path_type: "id" | "slug", path: Username | Reference, { avatars, detail }: {
    avatars?: boolean;
    detail?: "identity" | "info" | "profile";
} = {}, options?: RequestOptions): Promise<UserGetResponse> {
    return await _.unwrap(http.fetchJson(`/user/${path_type}/${path}${QS.query(QS.form({
        avatars,
        detail
    }))}`, {
        ...options
    }));
}
/**
 * Resets a user's profile.
 *
 * > This endpoint is only available to platform administrators.
 *
 */
async userResetProfile(path_type: "id" | "slug", path: Username | Reference, { avatars }: {
    avatars?: boolean;
} = {}, options?: RequestOptions): Promise<void> {
    return await _.unwrap(http.fetchVoid(`/user/${path_type}/${path}${QS.query(QS.form({
        avatars
    }))}`, {
        ...options,
        method: "DELETE"
    }));
}
/**
 * Gets a user's avatar. This won't return the avatar directly, but rather return the URL for it.
 */
async userGetAvatar(path_type: "id" | "slug", path: Username | Reference, options?: RequestOptions): Promise<AvatarURLObject> {
    return await _.unwrap(http.fetchJson(`/user/${path_type}/${path}/avatar`, {
        ...options
    }));
}
/**
 * Removes a user's avatar.
 *
 * > This endpoint is only available to platform administrators.
 *
 */
async userRemoveAvatar(path_type: "id" | "slug", path: Username | Reference, options?: RequestOptions): Promise<void> {
    return await _.unwrap(http.fetchVoid(`/user/${path_type}/${path}/avatar`, {
        ...options,
        method: "DELETE"
    }));
}
/**
 * Gets whether or not the client has a user blocked.
 */
async userGetBlocked(path_type: "id" | "slug", path: Username | Reference, options?: RequestOptions): Promise<UserBlocked> {
    return await _.unwrap(http.fetchJson(`/user/${path_type}/${path}/block`, {
        ...options
    }));
}
/**
 * Updates whether or not the client has a user blocked.
 */
async userUpdateBlocked(path_type: "id" | "slug", path: Username | Reference, userBlocked: UserBlocked, options?: RequestOptions): Promise<void> {
    return await _.unwrap(http.fetchVoid(`/user/${path_type}/${path}/block`, http.json({
        ...options,
        method: "PUT",
        body: userBlocked
    })));
}
/**
 * Gets the sites the client is a member of.
 */
async membershipGetList(options?: RequestOptions): Promise<MembershipList> {
    return await _.unwrap(http.fetchJson("/membership", {
        ...options
    }));
}
/**
 * Gets the sites the client has requested to join.
 */
async membershipGetApplications(options?: RequestOptions): Promise<ApplicationSendList> {
    return await _.unwrap(http.fetchJson("/membership/applications", {
        ...options
    }));
}
/**
 * Gets the sites the client has been invited to join.
 */
async membershipGetInvites(options?: RequestOptions): Promise<InviteList> {
    return await _.unwrap(http.fetchJson("/membership/invites", {
        ...options
    }));
}
/**
 * Gets the client membership status for a site.
 */
async membershipSiteGet(site: string, options?: RequestOptions): Promise<MembershipStatus> {
    return await _.unwrap(http.fetchJson(`/membership/site/${site}`, {
        ...options
    }));
}
/**
 * Requests to join a site (application).
 */
async membershipSiteApply(site: string, applicationSend: ApplicationSend, options?: RequestOptions): Promise<void> {
    return await _.unwrap(http.fetchVoid(`/membership/site/${site}`, http.json({
        ...options,
        method: "POST",
        body: applicationSend
    })));
}
/**
 * Leaves a site.
 */
async membershipSiteLeave(site: string, options?: RequestOptions): Promise<void> {
    return await _.unwrap(http.fetchVoid(`/membership/site/${site}`, {
        ...options,
        method: "DELETE"
    }));
}
/**
 * Gets the sites a user is a member of.
 */
async membershipUserGetList(path_type: "id" | "slug", path: Username | Reference, options?: RequestOptions): Promise<MembershipList> {
    return await _.unwrap(http.fetchJson(`/member/${path_type}/${path}/membership`, {
        ...options
    }));
}
/**
 * Gets a user's membership status for a site.
 */
async membershipUserSiteGet(site: string, path_type: "id" | "slug", path: Username | Reference, options?: RequestOptions): Promise<MembershipStatus> {
    return await _.unwrap(http.fetchJson(`/member/${path_type}/${path}/membership/${site}`, {
        ...options
    }));
}
/**
 * Gets the role of a user.
 */
async membershipUserGetRole(path_type: "id" | "slug", path: Username | Reference, options?: RequestOptions): Promise<MembershipRole> {
    return await _.unwrap(http.fetchJson(`/member/${path_type}/${path}/role`, {
        ...options
    }));
}
/**
 * Sets the role of a user.
 */
async membershipUserSetRole(path_type: "id" | "slug", path: Username | Reference, membershipRole: MembershipRole, options?: RequestOptions): Promise<void> {
    return await _.unwrap(http.fetchVoid(`/member/${path_type}/${path}/role`, http.json({
        ...options,
        method: "POST",
        body: membershipRole
    })));
}
/**
 * Invites a user to join a site.
 */
async membershipUserInvite(path_type: "id" | "slug", path: Username | Reference, inviteSend: InviteSend, options?: RequestOptions): Promise<void> {
    return await _.unwrap(http.fetchVoid(`/member/${path_type}/${path}/invite`, http.json({
        ...options,
        method: "POST",
        body: inviteSend
    })));
}
/**
 * Creates a new page.
 */
async pageCreate(pageCreateOptions: PageCreateOptions, options?: RequestOptions): Promise<void> {
    return await _.unwrap(http.fetchVoid("/page", http.json({
        ...options,
        method: "POST",
        body: pageCreateOptions
    })));
}
/**
 * Gets a page.
 */
async pageGet(path_type: "id" | "slug", path: Slug | Reference, { type, avatars }: {
    "type"?: "all" | "metadata" | "metadata-html" | "metadata-wikitext" | "wikitext" | "html" | "none";
    avatars?: boolean;
} = {}, options?: RequestOptions): Promise<PageGetResponse> {
    return await _.unwrap(http.fetchJson(`/page/${path_type}/${path}${QS.query(QS.form({
        type,
        avatars
    }))}`, {
        ...options
    }));
}
/**
 * Updates a page.
 */
async pageUpdate(path_type: "id" | "slug", path: Slug | Reference, pagePatch: PagePatch, options?: RequestOptions): Promise<void> {
    return await _.unwrap(http.fetchVoid(`/page/${path_type}/${path}`, http.json({
        ...options,
        method: "PATCH",
        body: pagePatch
    })));
}
/**
 * Deletes a page.
 */
async pageDelete(path_type: "id" | "slug", path: Slug | Reference, options?: RequestOptions): Promise<void> {
    return await _.unwrap(http.fetchVoid(`/page/${path_type}/${path}`, {
        ...options,
        method: "DELETE"
    }));
}
/**
 * Restores a previously deleted page.
 */
async pageRestore(id: Reference, slugObject: SlugObject, options?: RequestOptions): Promise<void> {
    return await _.unwrap(http.fetchVoid(`/page/id/${id}/restore`, http.json({
        ...options,
        method: "POST",
        body: slugObject
    })));
}
/**
 * Changes the path/slug/name of a page.
 */
async pageRename(path_type: "id" | "slug", path: Slug | Reference, slugObject: SlugObject, options?: RequestOptions): Promise<void> {
    return await _.unwrap(http.fetchVoid(`/page/${path_type}/${path}/rename`, http.json({
        ...options,
        method: "POST",
        body: slugObject
    })));
}
/**
 * Gets the update/revision history of a page.
 */
async revisionPageGetHistory(path_type: "id" | "slug", path: Slug | Reference, { cursor, limit, avatars }: {
    cursor?: number;
    limit?: number;
    avatars?: boolean;
} = {}, options?: RequestOptions): Promise<RevisionHistory> {
    return await _.unwrap(http.fetchJson(`/page/${path_type}/${path}/revision${QS.query(QS.form({
        cursor,
        limit,
        avatars
    }))}`, {
        ...options
    }));
}
/**
 * Gets the page corresponding to a revision.
 */
async revisionGet(path_type: "id" | "slug", path: Slug | Reference, revision: Reference, { type, avatars }: {
    "type"?: "all" | "metadata" | "metadata-html" | "metadata-wikitext" | "wikitext" | "html" | "none";
    avatars?: boolean;
} = {}, options?: RequestOptions): Promise<RevisionGetResponse> {
    return await _.unwrap(http.fetchJson(`/page/${path_type}/${path}/revision/${revision}${QS.query(QS.form({
        type,
        avatars
    }))}`, {
        ...options
    }));
}
/**
 * Updates the metadata of a revision.
 */
async revisionUpdateMetadata(path_type: "id" | "slug", path: Slug | Reference, revision: Reference, revisionPatch: RevisionPatch, options?: RequestOptions): Promise<void> {
    return await _.unwrap(http.fetchVoid(`/page/${path_type}/${path}/revision/${revision}`, http.json({
        ...options,
        method: "PATCH",
        body: revisionPatch
    })));
}
/**
 * Resets a page to a past revision.
 */
async revisionResetToRevision(path_type: "id" | "slug", path: Slug | Reference, revision: Reference, options?: RequestOptions): Promise<void> {
    return await _.unwrap(http.fetchVoid(`/page/${path_type}/${path}/revision/${revision}`, {
        ...options,
        method: "POST"
    }));
}
/**
 * Gets the tags of a page.
 */
async tagPageGet(path_type: "id" | "slug", path: Slug | Reference, options?: RequestOptions): Promise<TagListObject> {
    return await _.unwrap(http.fetchJson(`/page/${path_type}/${path}/tags`, {
        ...options
    }));
}
/**
 * Updates the tags of a page.
 */
async tagPageUpdate(path_type: "id" | "slug", path: Slug | Reference, tagListObject: TagListObject, options?: RequestOptions): Promise<void> {
    return await _.unwrap(http.fetchVoid(`/page/${path_type}/${path}/tags`, http.json({
        ...options,
        method: "PUT",
        body: tagListObject
    })));
}
/**
 * Gets the score of a page.
 */
async votePageGetScore(path_type: "id" | "slug", path: Slug | Reference, options?: RequestOptions): Promise<Score> {
    return await _.unwrap(http.fetchJson(`/page/${path_type}/${path}/score`, {
        ...options
    }));
}
/**
 * Gets the voters and votes of a page.
 */
async votePageGetVoters(path_type: "id" | "slug", path: Slug | Reference, { cursor, limit, avatars }: {
    cursor?: number;
    limit?: number;
    avatars?: boolean;
} = {}, options?: RequestOptions): Promise<VoterList> {
    return await _.unwrap(http.fetchJson(`/page/${path_type}/${path}/voters${QS.query(QS.form({
        cursor,
        limit,
        avatars
    }))}`, {
        ...options
    }));
}
/**
 * Gets the client's voting state on a page, if any.
 */
async votePageGet(path_type: "id" | "slug", path: Slug | Reference, options?: RequestOptions): Promise<CastVoteStatus> {
    return await _.unwrap(http.fetchJson(`/page/${path_type}/${path}/vote`, {
        ...options
    }));
}
/**
 * Updates/sets the client's voting state on a page.
 */
async votePageUpdateVote(path_type: "id" | "slug", path: Slug | Reference, castVoteObject: CastVoteObject, options?: RequestOptions): Promise<void> {
    return await _.unwrap(http.fetchVoid(`/page/${path_type}/${path}/vote`, http.json({
        ...options,
        method: "PUT",
        body: castVoteObject
    })));
}
/**
 * Removes the client's voting state on a page.
 */
async votePageRemoveVote(path_type: "id" | "slug", path: Slug | Reference, options?: RequestOptions): Promise<void> {
    return await _.unwrap(http.fetchVoid(`/page/${path_type}/${path}/vote`, {
        ...options,
        method: "DELETE"
    }));
}
/**
 * Gets metadata on all files attached to a page.
 */
async filePageGetMetadata(path_type: "id" | "slug", path: Slug | Reference, { avatars }: {
    avatars?: boolean;
} = {}, options?: RequestOptions): Promise<FileMetadataList> {
    return await _.unwrap(http.fetchJson(`/page/${path_type}/${path}/file${QS.query(QS.form({
        avatars
    }))}`, {
        ...options
    }));
}
/**
 * Adds a new file to a page.
 */
async filePageAdd(path_type: "id" | "slug", path: Slug | Reference, fileUpload: FileUpload, options?: RequestOptions): Promise<void> {
    return await _.unwrap(http.fetchVoid(`/page/${path_type}/${path}/file`, http.multipart({
        ...options,
        method: "POST",
        body: fileUpload
    })));
}
/**
 * Gets metadata on the files attached directly to the site instance.
 *
 * > This does not include files attached to _pages_.
 *
 */
async fileSiteGetMetadata({ avatars }: {
    avatars?: boolean;
} = {}, options?: RequestOptions): Promise<FileMetadataList> {
    return await _.unwrap(http.fetchJson(`/file${QS.query(QS.form({
        avatars
    }))}`, {
        ...options
    }));
}
/**
 * Adds a new file to a site instance.
 */
async fileSiteAdd(fileUpload: FileUpload, options?: RequestOptions): Promise<void> {
    return await _.unwrap(http.fetchVoid("/file", http.multipart({
        ...options,
        method: "POST",
        body: fileUpload
    })));
}
/**
 * Gets a file.
 */
async fileGet(id: Reference, options?: RequestOptions): Promise<string> {
    return await _.unwrap(http.fetch(`/file/${id}`, {
        ...options
    }));
}
/**
 * Deletes a file.
 */
async fileDelete(id: Reference, options?: RequestOptions): Promise<void> {
    return await _.unwrap(http.fetchVoid(`/file/${id}`, {
        ...options,
        method: "DELETE"
    }));
}
/**
 * Gets the site's file-system metadata, e.g. remaining file space.
 */
async fileGetSiteMetadata(options?: RequestOptions): Promise<FileSiteMetadata> {
    return await _.unwrap(http.fetchJson("/file/metadata", {
        ...options
    }));
}
/**
 * Gets a file's metadata.
 */
async fileGetMetadata(id: Reference, { avatars }: {
    avatars?: boolean;
} = {}, options?: RequestOptions): Promise<FileMetadata> {
    return await _.unwrap(http.fetchJson(`/file/${id}/metadata${QS.query(QS.form({
        avatars
    }))}`, {
        ...options
    }));
}
/**
 * Gets the reports against a user.
 */
async reportUserGet(path_type: "id" | "slug", path: Username | Reference, { avatars }: {
    avatars?: boolean;
} = {}, options?: RequestOptions): Promise<ReportList> {
    return await _.unwrap(http.fetchJson(`/user/${path_type}/${path}/report${QS.query(QS.form({
        avatars
    }))}`, {
        ...options
    }));
}
/**
 * Reports a user.
 */
async reportUserSend(path_type: "id" | "slug", path: Username | Reference, reportSend: ReportSend, options?: RequestOptions): Promise<void> {
    return await _.unwrap(http.fetchVoid(`/user/${path_type}/${path}/report`, http.json({
        ...options,
        method: "POST",
        body: reportSend
    })));
}
/**
 * Gets a page's reports.
 */
async reportPageGet(path_type: "id" | "slug", path: Slug | Reference, { avatars }: {
    avatars?: boolean;
} = {}, options?: RequestOptions): Promise<ReportList> {
    return await _.unwrap(http.fetchJson(`/page/${path_type}/${path}/report${QS.query(QS.form({
        avatars
    }))}`, {
        ...options
    }));
}
/**
 * Reports a page.
 */
async reportPageSend(path_type: "id" | "slug", path: Slug | Reference, reportSend: ReportSend, options?: RequestOptions): Promise<void> {
    return await _.unwrap(http.fetchVoid(`/page/${path_type}/${path}/report`, http.json({
        ...options,
        method: "POST",
        body: reportSend
    })));
}
/**
 * Gets a report.
 */
async reportGet(id: Reference, { avatars }: {
    avatars?: boolean;
} = {}, options?: RequestOptions): Promise<Report> {
    return await _.unwrap(http.fetchJson(`/report/${id}${QS.query(QS.form({
        avatars
    }))}`, {
        ...options
    }));
}
/**
 * Gets the reports against a site.
 *
 * > This endpoint is only available to platform administrators.
 *
 */
async abuseSiteGet({ avatars }: {
    avatars?: boolean;
} = {}, options?: RequestOptions): Promise<ReportList> {
    return await _.unwrap(http.fetchJson(`/abuse${QS.query(QS.form({
        avatars
    }))}`, {
        ...options
    }));
}
/**
 * Reports a site.
 */
async abuseSiteSend(reportSend: ReportSend, options?: RequestOptions): Promise<void> {
    return await _.unwrap(http.fetchVoid("/abuse", http.json({
        ...options,
        method: "POST",
        body: reportSend
    })));
}
/**
 * Gets the reports against a user.
 *
 * > This endpoint is only available to platform administrators.
 *
 */
async abuseUserGet(path_type: "id" | "slug", path: Username | Reference, { avatars }: {
    avatars?: boolean;
} = {}, options?: RequestOptions): Promise<ReportList> {
    return await _.unwrap(http.fetchJson(`/user/${path_type}/${path}/abuse${QS.query(QS.form({
        avatars
    }))}`, {
        ...options
    }));
}
/**
 * Reports a user.
 */
async abuseUserSend(path_type: "id" | "slug", path: Username | Reference, reportSend: ReportSend, options?: RequestOptions): Promise<void> {
    return await _.unwrap(http.fetchVoid(`/user/${path_type}/${path}/abuse`, http.json({
        ...options,
        method: "POST",
        body: reportSend
    })));
}
/**
 * Gets a page's reports.
 *
 * > This endpoint is only available to platform administrators.
 *
 */
async abusePageGet(path_type: "id" | "slug", path: Slug | Reference, { avatars }: {
    avatars?: boolean;
} = {}, options?: RequestOptions): Promise<ReportList> {
    return await _.unwrap(http.fetchJson(`/page/${path_type}/${path}/abuse${QS.query(QS.form({
        avatars
    }))}`, {
        ...options
    }));
}
/**
 * Reports a page.
 */
async abusePageSend(path_type: "id" | "slug", path: Slug | Reference, reportSend: ReportSend, options?: RequestOptions): Promise<void> {
    return await _.unwrap(http.fetchVoid(`/page/${path_type}/${path}/abuse`, http.json({
        ...options,
        method: "POST",
        body: reportSend
    })));
}
/**
 * Gets a report.
 *
 * > This endpoint is only available to platform administrators.
 *
 */
async abuseGet(id: Reference, { avatars }: {
    avatars?: boolean;
} = {}, options?: RequestOptions): Promise<Report> {
    return await _.unwrap(http.fetchJson(`/abuse/${id}${QS.query(QS.form({
        avatars
    }))}`, {
        ...options
    }));
}
/**
 * Gets all of the client's messages.
 */
async messageGetList({ cursor, limit, detail, archived, avatars }: {
    cursor?: number;
    limit?: number;
    detail?: "with-html" | "metadata";
    archived?: boolean;
    avatars?: boolean;
} = {}, options?: RequestOptions): Promise<MessageList> {
    return await _.unwrap(http.fetchJson(`/message${QS.query(QS.form({
        cursor,
        limit,
        detail,
        archived,
        avatars
    }))}`, {
        ...options
    }));
}
/**
 * Gets a message.
 */
async messageGet(id: Reference, { detail, avatars }: {
    detail?: "with-html" | "metadata";
    avatars?: boolean;
} = {}, options?: RequestOptions): Promise<Message> {
    return await _.unwrap(http.fetchJson(`/message/${id}${QS.query(QS.form({
        detail,
        avatars
    }))}`, {
        ...options
    }));
}
/**
 * Updates the metadata of a message, such as read or unread.
 */
async messageUpdate(id: Reference, messagePatch: MessagePatch, options?: RequestOptions): Promise<void> {
    return await _.unwrap(http.fetchVoid(`/message/${id}`, http.json({
        ...options,
        method: "PATCH",
        body: messagePatch
    })));
}
/**
 * Deletes a message.
 */
async messageDelete(id: Reference, options?: RequestOptions): Promise<void> {
    return await _.unwrap(http.fetchVoid(`/message/${id}`, {
        ...options,
        method: "DELETE"
    }));
}
/**
 * Messages a user.
 */
async messageSend(path_type: "id" | "slug", path: Username | Reference, messageSend: MessageSend, options?: RequestOptions): Promise<void> {
    return await _.unwrap(http.fetchVoid(`/user/${path_type}/${path}/message`, http.json({
        ...options,
        method: "POST",
        body: messageSend
    })));
}
/**
 * Gets the groups and categories of a forum.
 */
async forumGet({ avatars }: {
    avatars?: boolean;
} = {}, options?: RequestOptions): Promise<Forum> {
    return await _.unwrap(http.fetchJson(`/forum${QS.query(QS.form({
        avatars
    }))}`, {
        ...options
    }));
}
/**
 * Gets the groups of a forum.
 */
async forumGroupGetList({ avatars }: {
    avatars?: boolean;
} = {}, options?: RequestOptions): Promise<ForumGroupList> {
    return await _.unwrap(http.fetchJson(`/forum/group${QS.query(QS.form({
        avatars
    }))}`, {
        ...options
    }));
}
/**
 * Gets a group.
 */
async forumGroupGet(id: Reference, { avatars }: {
    avatars?: boolean;
} = {}, options?: RequestOptions): Promise<ForumGroup> {
    return await _.unwrap(http.fetchJson(`/forum/group/${id}${QS.query(QS.form({
        avatars
    }))}`, {
        ...options
    }));
}
/**
 * Updates a group.
 */
async forumGroupUpdate(id: Reference, forumGroupPatch: ForumGroupPatch, options?: RequestOptions): Promise<void> {
    return await _.unwrap(http.fetchVoid(`/forum/group/${id}`, http.json({
        ...options,
        method: "PATCH",
        body: forumGroupPatch
    })));
}
/**
 * Creates a new category inside of a group.
 */
async forumGroupAddCategory(id: Reference, forumCategoryCreate: ForumCategoryCreate, options?: RequestOptions): Promise<void> {
    return await _.unwrap(http.fetchVoid(`/forum/group/${id}`, http.json({
        ...options,
        method: "POST",
        body: forumCategoryCreate
    })));
}
/**
 * Deletes a group.
 */
async forumGroupDelete(id: Reference, options?: RequestOptions): Promise<void> {
    return await _.unwrap(http.fetchVoid(`/forum/group/${id}`, {
        ...options,
        method: "DELETE"
    }));
}
/**
 * Gets the categories of a group.
 */
async forumGroupGetCategories(id: Reference, { avatars }: {
    avatars?: boolean;
} = {}, options?: RequestOptions): Promise<ForumCategoryList> {
    return await _.unwrap(http.fetchJson(`/forum/group/${id}/categories${QS.query(QS.form({
        avatars
    }))}`, {
        ...options
    }));
}
/**
 * Gets the categories of a forum.
 */
async forumCategoryGetList({ avatars }: {
    avatars?: boolean;
} = {}, options?: RequestOptions): Promise<ForumCategoryList> {
    return await _.unwrap(http.fetchJson(`/forum/category${QS.query(QS.form({
        avatars
    }))}`, {
        ...options
    }));
}
/**
 * Gets a category.
 */
async forumCategoryGet(id: Reference, { avatars }: {
    avatars?: boolean;
} = {}, options?: RequestOptions): Promise<ForumCategory> {
    return await _.unwrap(http.fetchJson(`/forum/category/${id}${QS.query(QS.form({
        avatars
    }))}`, {
        ...options
    }));
}
/**
 * Updates a category.
 */
async forumCategoryUpdate(id: Reference, forumCategoryPatch: ForumCategoryPatch, options?: RequestOptions): Promise<void> {
    return await _.unwrap(http.fetchVoid(`/forum/category/${id}`, http.json({
        ...options,
        method: "PATCH",
        body: forumCategoryPatch
    })));
}
/**
 * Creates a new thread inside of a category.
 */
async forumCategoryAddThread(id: Reference, forumThreadCreate: ForumThreadCreate, options?: RequestOptions): Promise<void> {
    return await _.unwrap(http.fetchVoid(`/forum/category/${id}`, http.json({
        ...options,
        method: "POST",
        body: forumThreadCreate
    })));
}
/**
 * Deletes a category.
 */
async forumCategoryDelete(id: Reference, options?: RequestOptions): Promise<void> {
    return await _.unwrap(http.fetchVoid(`/forum/category/${id}`, {
        ...options,
        method: "DELETE"
    }));
}
/**
 * Gets the threads of a category.
 */
async forumCategoryGetThreads(id: Reference, { cursor, limit, avatars }: {
    cursor?: number;
    limit?: number;
    avatars?: boolean;
} = {}, options?: RequestOptions): Promise<ForumThreadList> {
    return await _.unwrap(http.fetchJson(`/forum/category/${id}/threads${QS.query(QS.form({
        cursor,
        limit,
        avatars
    }))}`, {
        ...options
    }));
}
/**
 * Gets a thread.
 */
async forumThreadGet(id: Reference, { avatars }: {
    avatars?: boolean;
} = {}, options?: RequestOptions): Promise<ForumThread> {
    return await _.unwrap(http.fetchJson(`/forum/thread/${id}${QS.query(QS.form({
        avatars
    }))}`, {
        ...options
    }));
}
/**
 * Updates a thread.
 */
async forumThreadUpdate(id: Reference, forumThreadPatch: ForumThreadPatch, options?: RequestOptions): Promise<void> {
    return await _.unwrap(http.fetchVoid(`/forum/thread/${id}`, http.json({
        ...options,
        method: "PATCH",
        body: forumThreadPatch
    })));
}
/**
 * Creates a new post inside of a thread.
 */
async forumThreadAddPost(id: Reference, forumPostCreate: ForumPostCreate, options?: RequestOptions): Promise<void> {
    return await _.unwrap(http.fetchVoid(`/forum/thread/${id}`, http.json({
        ...options,
        method: "POST",
        body: forumPostCreate
    })));
}
/**
 * Deletes a thread.
 */
async forumThreadDelete(id: Reference, { permanent }: {
    permanent?: boolean;
} = {}, options?: RequestOptions): Promise<void> {
    return await _.unwrap(http.fetchVoid(`/forum/thread/${id}${QS.query(QS.form({
        permanent
    }))}`, {
        ...options,
        method: "DELETE"
    }));
}
/**
 * Gets the posts of a thread.
 */
async forumThreadGetPosts(id: Reference, { cursor, limit, detail, depth, avatars }: {
    cursor?: number;
    limit?: number;
    detail?: "none" | "metadata" | "with-html" | "full";
    depth?: number;
    avatars?: boolean;
} = {}, options?: RequestOptions): Promise<ForumPostList> {
    return await _.unwrap(http.fetchJson(`/forum/thread/${id}/posts${QS.query(QS.form({
        cursor,
        limit,
        detail,
        depth,
        avatars
    }))}`, {
        ...options
    }));
}
/**
 * Gets a post.
 */
async forumPostGet(id: Reference, { detail, depth, avatars }: {
    detail?: "none" | "metadata" | "with-html" | "full";
    depth?: number;
    avatars?: boolean;
} = {}, options?: RequestOptions): Promise<ForumPost> {
    return await _.unwrap(http.fetchJson(`/forum/post/${id}${QS.query(QS.form({
        detail,
        depth,
        avatars
    }))}`, {
        ...options
    }));
}
/**
 * Updates a post.
 */
async forumPostUpdate(id: Reference, forumPostPatch: ForumPostPatch, options?: RequestOptions): Promise<void> {
    return await _.unwrap(http.fetchVoid(`/forum/post/${id}`, http.json({
        ...options,
        method: "PATCH",
        body: forumPostPatch
    })));
}
/**
 * Replies to a post with another post.
 */
async forumPostReply(id: Reference, forumPostCreate: ForumPostCreate, options?: RequestOptions): Promise<void> {
    return await _.unwrap(http.fetchVoid(`/forum/post/${id}`, http.json({
        ...options,
        method: "POST",
        body: forumPostCreate
    })));
}
/**
 * Deletes a post.
 */
async forumPostDelete(id: Reference, { permanent }: {
    permanent?: boolean;
} = {}, options?: RequestOptions): Promise<void> {
    return await _.unwrap(http.fetchVoid(`/forum/post/${id}${QS.query(QS.form({
        permanent
    }))}`, {
        ...options,
        method: "DELETE"
    }));
}
/**
 * Gets the replies to a post.
 */
async forumPostGetReplies(id: Reference, { cursor, limit, detail, depth, avatars }: {
    cursor?: number;
    limit?: number;
    detail?: "none" | "metadata" | "with-html" | "full";
    depth?: number;
    avatars?: boolean;
} = {}, options?: RequestOptions): Promise<ForumPostList> {
    return await _.unwrap(http.fetchJson(`/forum/post/${id}/replies${QS.query(QS.form({
        cursor,
        limit,
        detail,
        depth,
        avatars
    }))}`, {
        ...options
    }));
}
/**
 * Gets the update/revision history of a post.
 */
async forumPostRevisionGetHistory(id: Reference, { cursor, limit, avatars }: {
    cursor?: number;
    limit?: number;
    avatars?: boolean;
} = {}, options?: RequestOptions): Promise<RevisionHistory> {
    return await _.unwrap(http.fetchJson(`/forum/post/${id}/revision${QS.query(QS.form({
        cursor,
        limit,
        avatars
    }))}`, {
        ...options
    }));
}
/**
 * Gets the post corresponding to a revision.
 */
async forumPostRevisionGet(id: Reference, revision: Reference, { avatars, detail }: {
    avatars?: boolean;
    detail?: "none" | "metadata" | "with-html" | "full";
} = {}, options?: RequestOptions): Promise<ForumPost> {
    return await _.unwrap(http.fetchJson(`/forum/post/${id}/revision/${revision}${QS.query(QS.form({
        avatars,
        detail
    }))}`, {
        ...options
    }));
}
/**
 * Updates the metadata of a revision.
 */
async forumPostRevisionUpdateMetadata(id: Reference, revision: Reference, revisionPatch: RevisionPatch, options?: RequestOptions): Promise<void> {
    return await _.unwrap(http.fetchVoid(`/forum/post/${id}/revision/${revision}`, http.json({
        ...options,
        method: "PATCH",
        body: revisionPatch
    })));
}
/**
 * Resets a forum post to a past revision.
 */
async forumPostResetToRevision(id: Reference, revision: Reference, options?: RequestOptions): Promise<void> {
    return await _.unwrap(http.fetchVoid(`/forum/post/${id}/revision/${revision}`, {
        ...options,
        method: "POST"
    }));
}
/**
 * Kicks a user from a site.
 */
async moderationKick(path_type: "id" | "slug", path: Username | Reference, userKick: UserKick, options?: RequestOptions): Promise<void> {
    return await _.unwrap(http.fetchVoid(`/user/${path_type}/${path}/kick`, http.json({
        ...options,
        method: "PUT",
        body: userKick
    })));
}
/**
 * Gets the list of users banned from a site.
 */
async moderationBanGetList(options?: RequestOptions): Promise<UserBannedList> {
    return await _.unwrap(http.fetchJson("/moderation/banned", {
        ...options
    }));
}
/**
 * Gets if a user is banned.
 */
async moderationBanGet(path_type: "id" | "slug", path: Username | Reference, options?: RequestOptions): Promise<UserBanned> {
    return await _.unwrap(http.fetchJson(`/user/${path_type}/${path}/ban`, {
        ...options
    }));
}
/**
 * Bans a user. Providing `null` for `until` describes a perma-ban.
 */
async moderationBan(path_type: "id" | "slug", path: Username | Reference, userBan: UserBan, options?: RequestOptions): Promise<void> {
    return await _.unwrap(http.fetchVoid(`/user/${path_type}/${path}/ban`, http.json({
        ...options,
        method: "PUT",
        body: userBan
    })));
}
/**
 * Unbans a user, if they were banned to begin with.
 */
async moderationUnban(path_type: "id" | "slug", path: Username | Reference, options?: RequestOptions): Promise<void> {
    return await _.unwrap(http.fetchVoid(`/user/${path_type}/${path}/ban`, {
        ...options,
        method: "DELETE"
    }));
}
/**
 * Gets the list of categories on a site.
 */
async categoryGetList(options?: RequestOptions): Promise<CategoryList> {
    return await _.unwrap(http.fetchJson("/category", {
        ...options
    }));
}
/**
 * Gets the default category.
 */
async categoryDefaultGet(options?: RequestOptions): Promise<CategoryDefault> {
    return await _.unwrap(http.fetchJson("/category/default", {
        ...options
    }));
}
/**
 * Update (patch) the default category.
 */
async categoryDefaultPatch(categoryDefaultPatch: CategoryDefaultPatch, options?: RequestOptions): Promise<void> {
    return await _.unwrap(http.fetchVoid("/category/default", http.json({
        ...options,
        method: "PATCH",
        body: categoryDefaultPatch
    })));
}
/**
 * Gets a category.
 */
async categoryGet(id: Reference, options?: RequestOptions): Promise<Category> {
    return await _.unwrap(http.fetchJson(`/category/id/${id}`, {
        ...options
    }));
}
/**
 * Update (patch) a category.
 */
async categoryPatch(id: Reference, categoryPatch: CategoryPatch, options?: RequestOptions): Promise<void> {
    return await _.unwrap(http.fetchVoid(`/category/id/${id}`, http.json({
        ...options,
        method: "PATCH",
        body: categoryPatch
    })));
}
/**
 * Gets the site's settings.
 */
async siteSettingsGet(options?: RequestOptions): Promise<SiteSettings> {
    return await _.unwrap(http.fetchJson("/site/settings", {
        ...options
    }));
}
/**
 * Update (patch) the site's settings.
 */
async siteSettingsPatch(siteSettingsPatch: SiteSettingsPatch, options?: RequestOptions): Promise<void> {
    return await _.unwrap(http.fetchVoid("/site/settings", http.json({
        ...options,
        method: "PATCH",
        body: siteSettingsPatch
    })));
}
/**
 * Gets the site's pending applications.
 */
async siteApplicationGetList({ cursor, limit }: {
    cursor?: number;
    limit?: number;
} = {}, options?: RequestOptions): Promise<ApplicationList> {
    return await _.unwrap(http.fetchJson(`/site/application${QS.query(QS.form({
        cursor,
        limit
    }))}`, {
        ...options
    }));
}
/**
 * Gets an application.
 */
async siteApplicationGet(id: Reference, options?: RequestOptions): Promise<Application> {
    return await _.unwrap(http.fetchJson(`/site/application/${id}`, {
        ...options
    }));
}
/**
 * Accepts an application.
 */
async siteApplicationAccept(id: Reference, options?: RequestOptions): Promise<void> {
    return await _.unwrap(http.fetchVoid(`/site/application/${id}`, {
        ...options,
        method: "POST"
    }));
}
/**
 * Rejects an application.
 */
async siteApplicationReject(id: Reference, options?: RequestOptions): Promise<void> {
    return await _.unwrap(http.fetchVoid(`/site/application/${id}`, {
        ...options,
        method: "DELETE"
    }));
}
/**
 * Gets a backup of the site.
 */
async siteBackupGet(options?: RequestOptions): Promise<string> {
    return await _.unwrap(http.fetch("/site/backup", {
        ...options
    }));
}
/**
 * Creates a new site.
 */
async siteCreate(createSiteSettings: CreateSiteSettings, options?: RequestOptions): Promise<void> {
    return await _.unwrap(http.fetchVoid("/site/create", http.json({
        ...options,
        method: "POST",
        body: createSiteSettings
    })));
}
/**
 * Starts the deletion process for the site. Requires additional email validation for the process to complete.
 *
 */
async siteRequestDeletion(options?: RequestOptions): Promise<void> {
    return await _.unwrap(http.fetchVoid("/site/request-deletion", {
        ...options,
        method: "POST"
    }));
}
/**
 * Gets the site's current notifications.
 */
async siteNotificationGet(options?: RequestOptions): Promise<NotificationList> {
    return await _.unwrap(http.fetchJson("/site/notification", {
        ...options
    }));
}
/**
 * Dismisses all of the site's notifications.
 */
async siteNotificationDismissAll(options?: RequestOptions): Promise<void> {
    return await _.unwrap(http.fetchVoid("/site/notification", {
        ...options,
        method: "DELETE"
    }));
}
/**
 * Sends a site newsletter.
 */
async siteNewsletterSend(siteNewsletter: SiteNewsletter, options?: RequestOptions): Promise<void> {
    return await _.unwrap(http.fetchVoid("/site/newsletter", http.json({
        ...options,
        method: "POST",
        body: siteNewsletter
    })));
}
/**
 * Transfers the site master-admin status to another user.
 */
async siteTransfer(siteTransfer: SiteTransfer, options?: RequestOptions): Promise<void> {
    return await _.unwrap(http.fetchVoid("/site/transfer", http.json({
        ...options,
        method: "POST",
        body: siteTransfer
    })));
}
}
