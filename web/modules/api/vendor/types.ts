export interface APIError {
    error: string;
}
export type Reference = number;
export type ReferenceTypes = "user" | "page" | "message" | "file" | "report" | "abuse" | "forum-group" | "forum-category" | "forum-thread" | "forum-post";
export interface ReferenceTypesObject {
    "type": ReferenceTypes;
}
export type Email = string;
export type Username = string;
export type LoginSpecifier = Email | Username;
export type Password = string;
export interface LoginOptions {
    login: LoginSpecifier;
    password: Password;
    remember?: boolean;
}
export interface CSRF {
    csrf: string;
}
export interface PasswordObject {
    password: Password;
}
export interface SessionState {
    sessionValid: boolean;
    authed: boolean;
}
export interface RegisterOptions {
    username: Username;
    email: Email;
    password: Password;
}
export interface EmailObject {
    email: Email;
}
export interface UpdateEmail {
    oldEmail: Email;
    newEmail: Email;
}
export interface UpdatePassword {
    oldPassword: Password;
    newPassword: Password;
}
export interface UsernameObject {
    username: Username;
}
export interface AccountSettings {
    acceptsInvites: boolean;
    language: string;
    allowMessages: "registered" | "co-members" | "nobody";
}
export interface AccountSettingsPatch {
    acceptsInvites?: boolean;
    language?: string;
    allowMessages?: "registered" | "co-members" | "nobody";
}
export interface Notification {
    level: "trivial" | "info" | "important" | "error";
    "type": "account" | "pm" | "site" | "forum" | "page" | "other";
    name: string;
    source: string;
    time: string;
    payload: {
        message: string;
    };
}
export interface NotificationList {
    notifications: Notification[];
}
export type Base64 = string;
export type UserRole = "guest" | "registered" | "member" | "moderator" | "admin" | "master-admin" | "platform-admin";
export interface UserIdentity {
    id: Reference;
    username: Username;
    tinyavatar: Base64 | null;
    karma: number;
    role: UserRole;
}
export type HTML = string;
export interface UserInfo extends UserIdentity {
    about: string | null;
    avatar: string | null;
    signature: HTML | null;
    since: string;
    lastActive: string;
}
export interface UserProfile extends UserInfo {
    realname: string | null;
    pronouns: string | null;
    birthday: string | null;
    location: string | null;
    links: {
        [key: string]: string;
    };
}
export type UserClientGetResponse = UserIdentity | UserInfo | UserProfile;
export type Wikitext = string;
export interface UserProfilePatch {
    about?: string | null;
    signature?: Wikitext | null;
    realname?: string | null;
    pronouns?: string | null;
    birthday?: string | null;
    location?: string | null;
    links?: {
        [key: string]: string;
    };
}
export type AvatarURL = string;
export interface AvatarURLObject {
    avatar: AvatarURL;
}
export interface UserBlockedList {
    users: UserIdentity[];
}
export type UserGetResponse = UserIdentity | UserInfo | UserProfile;
export interface UserBlocked {
    blocked: boolean;
}
export type SiteName = string;
export interface Membership {
    site: SiteName;
    role: UserRole;
}
export interface MembershipList {
    memberships: Membership[];
}
export interface ApplicationSendList {
    applications: {
        site: SiteName;
        message: string;
    }[];
}
export interface Invite {
    sender: UserIdentity;
    site: SiteName;
    message: string;
    time: string;
}
export interface InviteList {
    invites: Invite[];
}
export interface MembershipStatus {
    status: Membership | null;
}
export interface ApplicationSend {
    message: string;
}
export interface MembershipRole {
    role: "member" | "moderator" | "admin";
}
export interface InviteSend {
    site: SiteName;
    message: string;
}
export type Slug = string;
export interface PageCreateOptions {
    slug: Slug;
    title?: string;
    wikitext?: Wikitext;
}
export type TagList = string[];
export interface Page {
    id: Reference;
    slug: Slug;
    category: Reference;
    parent: Slug | null;
    children: Slug[];
    title: string;
    altTitle: string | null;
    tags: TagList;
    score: number;
    revision: number;
    created: string;
    creator: UserIdentity;
    updated: string;
    updater: UserIdentity;
    html?: HTML;
    wikitext?: Wikitext;
}
export interface WikitextObject {
    wikitext: Wikitext;
}
export interface HTMLObject {
    html: HTML;
}
export interface FTMLSyntaxTree {
    [key: string]: any;
}
export type PageGetResponse = Page | WikitextObject | HTMLObject | FTMLSyntaxTree;
export interface PagePatch {
    title?: string;
    wikitext?: Wikitext;
}
export interface SlugObject {
    slug: Slug;
}
export interface Paginated {
    pagination: {
        cursor: number;
        limit: number;
        pages: number;
    };
}
export interface Revision {
    revision: number;
    updated: string;
    updater: UserIdentity;
    hidden: boolean;
    message: string;
    flags: ("created" | "content" | "file" | "title" | "revert" | "tag" | "slug")[];
}
export interface RevisionHistory extends Paginated {
    revisions: number;
    history: Revision[];
}
export type RevisionGetResponse = Page | WikitextObject | HTMLObject | FTMLSyntaxTree;
export interface RevisionPatch {
    hidden?: boolean;
    message?: string;
}
export interface TagListObject {
    tags: TagList;
}
export type Score = {
    format: "plus";
    score: number;
    count: number;
    totals: {
        "0": number;
        "1": number;
    };
} | {
    format: "plusminus";
    score: number;
    count: number;
    totals: {
        "0": number;
        "1": number;
        "-1": number;
    };
} | {
    format: "star";
    score: number;
    count: number;
    totals: {
        "1": number;
        "2": number;
        "3": number;
        "4": number;
        "5": number;
    };
};
export type CastVotePlus = 0 | 1;
export type CastVotePlusMinus = -1 | 0 | 1;
export type CastVoteStar = 1 | 2 | 3 | 4 | 5;
export type Vote = UserIdentity & ({
    format: "plus";
    time: string;
    vote: CastVotePlus;
} | {
    format: "plusminus";
    time: string;
    vote: CastVotePlusMinus;
} | {
    format: "star";
    time: string;
    vote: CastVoteStar;
});
export interface VoterList extends Paginated {
    score: Score;
    voters: Vote[];
}
export type CastVote = CastVotePlusMinus | CastVoteStar;
export interface CastVoteStatus {
    vote: CastVote | null;
}
export interface CastVoteObject {
    vote: CastVote;
}
export interface Mime {
    "type": string;
    description: string;
}
export interface FileMetadata {
    id: Reference;
    size: number;
    comment: string;
    mime: Mime;
    uploader: UserIdentity;
    uploaded: string;
    url: string;
}
export interface FileMetadataList extends Paginated {
    files: FileMetadata[];
}
export type BinaryData = string;
export interface FileUpload {
    filename: string;
    comment: string;
    content: BinaryData;
}
export interface FileSiteMetadata {
    max: number;
    used: number;
    count: number;
    available: number;
}
export interface Report {
    id: Reference;
    target?: Reference;
    sender: UserIdentity;
    reason: string;
    time: string;
}
export interface ReportList {
    reports: Report[];
}
export interface ReportSend {
    reason: string;
}
export interface Message {
    id: Reference;
    read: boolean;
    archived?: boolean;
    time: string;
    "from": UserIdentity;
    subject: string;
    html?: HTML;
}
export interface MessageList extends Paginated {
    messages: Message[];
}
export interface MessagePatch {
    read?: boolean;
    archived?: boolean;
}
export interface MessageSend {
    subject: string;
    wikitext: Wikitext;
}
export interface ForumCreationContext {
    by: UserIdentity;
    time: string;
}
export interface ForumCategory {
    id: Reference;
    group: Reference;
    title: string;
    summary: string;
    threadCount: number;
    postCount: number;
    last: ForumCreationContext;
    permissions?: {
        createPosts: ("guest" | "registered" | "member")[];
        createThreads: ("guest" | "registered" | "member")[];
        edit: ("guest" | "registered" | "member" | "author")[];
    };
}
export interface ForumGroup {
    id: Reference;
    title: string;
    summary: string;
    categories: ForumCategory[];
}
export interface Forum {
    threadCount: number;
    postCount: number;
    groups: ForumGroup[];
}
export interface ForumGroupList {
    groups: ForumGroup[];
}
export interface ForumGroupPatch {
    title?: string;
    summary?: string;
    order?: Reference[];
}
export interface ForumCategoryCreate {
    title: string;
    summary?: string;
}
export interface ForumCategoryList {
    categories: ForumCategory[];
}
export interface ForumCategoryPatch {
    title?: string;
    summary?: string;
}
export interface ForumThreadCreate {
    title: string;
    summary?: string;
    stickied?: boolean;
    locked?: boolean;
}
export type ForumSortingTypes = "newest" | "oldset";
export interface ForumThread {
    id: Reference;
    category: Reference;
    group: Reference;
    title: string;
    stickied: boolean;
    locked: boolean;
    postCount: number;
    created: ForumCreationContext;
    last: ForumCreationContext;
}
export interface ForumThreadList extends Paginated {
    order: ForumSortingTypes;
    threads: ForumThread[];
}
export interface ForumThreadPatch {
    title?: string;
    summary?: string;
    stickied?: boolean;
    locked?: boolean;
}
export interface ForumPostCreate {
    title: string;
    wikitext: Wikitext;
}
export interface ForumPost {
    id: Reference;
    category: Reference;
    group: Reference;
    thread: Reference;
    parent: Reference | null;
    created: ForumCreationContext;
    revision: number;
    replyCount: number;
    html?: HTML;
    wikitext?: Wikitext;
    replies?: ForumPostList;
}
export interface ForumPostList extends Paginated {
    order: ForumSortingTypes;
    posts: ForumPost[];
}
export interface ForumPostPatch {
    title?: string;
    wikitext?: Wikitext;
}
export interface UserKick {
    reason: string;
}
export interface UserBanned {
    user: UserIdentity;
    when: string;
    until: string | null;
    reason: string;
}
export interface UserBannedList extends Paginated {
    banned: UserBanned[];
}
export interface UserBan {
    until: string | null;
    reason: string;
}
export interface Category {
    id: Reference;
    name: string;
    license: string | null;
    ratings: "disabled" | "plus" | "plusminus" | "star";
    discussions: boolean | null;
    autonumber: boolean;
    permissions: {
        createPages: ("guest" | "registered" | "member")[];
        renamePages: ("guest" | "registered" | "member" | "creator")[];
        deletePages: ("guest" | "registered" | "member" | "creator")[];
        uploadFiles: ("guest" | "registered" | "member" | "creator")[];
        changeFiles: ("guest" | "registered" | "member" | "creator")[];
        showOptions: ("guest" | "registered" | "member" | "creator")[];
        edit: ("guest" | "registered" | "member" | "creator")[];
    };
}
export interface CategoryList extends Paginated {
    categories: Category[];
}
export interface CategoryDefault {
    id: Reference;
    name: "_default";
    license: string;
    ratings: "disabled" | "plus" | "plusminus" | "star";
    discussions: boolean;
    autonumber: false;
    permissions: {
        createPages: ("guest" | "registered" | "member")[];
        renamePages: ("guest" | "registered" | "member" | "creator")[];
        deletePages: ("guest" | "registered" | "member" | "creator")[];
        uploadFiles: ("guest" | "registered" | "member" | "creator")[];
        changeFiles: ("guest" | "registered" | "member" | "creator")[];
        showOptions: ("guest" | "registered" | "member" | "creator")[];
        edit: ("guest" | "registered" | "member" | "creator")[];
    };
}
export interface CategoryDefaultPatch {
    license?: string;
    ratings?: "disabled" | "plus" | "plusminus" | "star";
    discussions?: boolean;
    autonumber?: false;
    permissions?: {
        createPages?: ("guest" | "registered" | "member")[];
        renamePages?: ("guest" | "registered" | "member" | "creator")[];
        deletePages?: ("guest" | "registered" | "member" | "creator")[];
        uploadFiles?: ("guest" | "registered" | "member" | "creator")[];
        changeFiles?: ("guest" | "registered" | "member" | "creator")[];
        showOptions?: ("guest" | "registered" | "member" | "creator")[];
        edit?: ("guest" | "registered" | "member" | "creator")[];
    };
}
export interface CategoryPatch {
    license?: string | null;
    ratings?: "disabled" | "plus" | "plusminus" | "star";
    discussions?: boolean | null;
    autonumber?: boolean;
    permissions?: {
        createPages?: ("guest" | "registered" | "member")[];
        renamePages?: ("guest" | "registered" | "member" | "creator")[];
        deletePages?: ("guest" | "registered" | "member" | "creator")[];
        uploadFiles?: ("guest" | "registered" | "member" | "creator")[];
        changeFiles?: ("guest" | "registered" | "member" | "creator")[];
        showOptions?: ("guest" | "registered" | "member" | "creator")[];
        edit?: ("guest" | "registered" | "member" | "creator")[];
    };
}
export interface SiteSettings {
    general: {
        address: string;
        title: string;
        subtitle: string;
        language: string;
        description: string;
        defaultPage: Slug;
        welcomePage: Slug;
    };
    integrations: {
        googleAnalytics: string | null;
    };
    security: {
        policy: "open";
        cloning: boolean;
        fileHotLinking: boolean;
    } | {
        policy: "closed";
        cloning: boolean;
        fileHotLinking: boolean;
        usersCanApply: boolean;
        sitePassword: string;
    } | {
        policy: "private";
        usersCanApply: boolean;
        sitePassword: string;
        guestDefaultPage: Slug;
        guestHideNav: boolean;
        extraUsers: UserIdentity[];
    };
    appearance: {
        userKarma: boolean;
        toolbar: {
            top: boolean;
            bottom: boolean;
        };
    };
    forum: {
        nestingDepth: number;
        permissions: {
            createPosts: ("guest" | "registered" | "member")[];
            createThreads: ("guest" | "registered" | "member")[];
            edit: ("guest" | "registered" | "member" | "author")[];
        };
    };
}
export interface SiteSettingsPatch {
    general?: {
        address?: string;
        title?: string;
        subtitle?: string;
        language?: string;
        description?: string;
        defaultPage?: Slug;
        welcomePage?: Slug;
    };
    integrations?: {
        googleAnalytics?: string | null;
    };
    security?: {
        policy?: "open";
        cloning?: boolean;
        fileHotLinking?: boolean;
    } | {
        policy?: "closed";
        cloning?: boolean;
        fileHotLinking?: boolean;
        usersCanApply?: boolean;
        sitePassword?: string;
    } | {
        policy?: "private";
        usersCanApply?: boolean;
        sitePassword?: string;
        guestDefaultPage?: Slug;
        guestHideNav?: boolean;
        extraUsers?: Reference[];
    };
    appearance?: {
        userKarma?: boolean;
        toolbar?: {
            top?: boolean;
            bottom?: boolean;
        };
    };
    forum?: {
        nestingDepth?: number;
        permissions?: {
            createPosts?: ("guest" | "registered" | "member")[];
            createThreads?: ("guest" | "registered" | "member")[];
            edit?: ("guest" | "registered" | "member" | "author")[];
        };
    };
}
export interface Application {
    id: Reference;
    sender: UserIdentity;
    message: string;
    time: string;
}
export interface ApplicationList extends Paginated {
    applications: Application[];
}
export interface CreateSiteSettings {
    address: string;
    title: string;
    subtitle: string;
    language: string;
    description: string;
    defaultPage: Slug;
    welcomePage: Slug;
    policy: "open" | "closed" | "private";
}
export interface SiteNewsletter {
    to: ("member" | "moderator" | "admin")[];
    title: string;
    wikitext: Wikitext;
}
export interface SiteTransfer {
    site: SiteName;
    current: Reference;
    next: Reference;
}
