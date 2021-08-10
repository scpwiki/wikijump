<?php

namespace Wikijump\Models;

use Illuminate\Database\Eloquent\Builder;
use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model;
use Wikijump\Traits\UsesBitmasks;

/**
 * Private (user-to-user) Messages
 * @package Wikijump\Models
 */
class UserMessage extends Model
{
    use HasFactory;
    use UsesBitmasks;

    /** Various flags to set on a PM. */
    public const MESSAGE_READ = 1;
    public const MESSAGE_DRAFT = 2;
    public const MESSAGE_STARRED = 4;
    public const MESSAGE_ARCHIVED = 8;
    # const MESSAGE_RESERVED = 16;
    # const MESSAGE_RESERVED = 32;
    # const MESSAGE_RESERVED = 64;
    # const MESSAGE_RESERVED = 128;

    /**
     * Mark a message as read.
     * @return bool
     */
    public function markAsRead() : bool
    {
        return $this->setFlag(self::MESSAGE_READ);
    }

    /**
     * Mark a message as unread.
     * @return bool
     */
    public function markAsUnread() : bool
    {
        return $this->clearFlag(self::MESSAGE_READ);
    }

    /**
     * Mark a message as a draft.
     * The `to_user_id` field is nullable to allow for this.
     * @return bool
     */
    public function markAsDraft() : bool
    {
        return $this->setFlag(self::MESSAGE_DRAFT);
    }

    /**
     * Unmark a message as a draft.
     * In practice, this is the point at which a message is "sent" and viewable
     *  to the other party, and visible in a user's "Sent" items instead of an "outbox."
     * @return bool
     */
    public function unmarkAsDraft() : bool
    {
        return $this->clearFlag(self::MESSAGE_DRAFT);
    }

    /**
     * Star/favorite a message.
     * @return bool
     */
    public function starMessage() : bool
    {
        return $this->setFlag(self::MESSAGE_STARRED);
    }

    /**
     * Unstar/unfavorite a message.
     * @return bool
     */
    public function unstarMessage() : bool
    {
        return $this->clearFlag(self::MESSAGE_STARRED);
    }

    /**
     * Archive a message, to get it out of the inbox without deleting it.
     * @return bool
     */
    public function archiveMessage() : bool
    {
        return $this->setFlag(self::MESSAGE_ARCHIVED);
    }

    /**
     * Move a message from the archive back to the inbox.
     * @return bool
     */
    public function unarchiveMessage() : bool
    {
        return $this->clearFlag(self::MESSAGE_ARCHIVED);
    }

    /**
     * A slight primer on scopes and bitwise operators.
     * Scopes can be used in combination with Query Builder.
     * e.g., the scopeUnread method for this class can be called via:
     * User::messages()->unread();
     * The `scope` prefix is omitted and Laravel handles the rest.
     * See https://laravel.com/docs/master/eloquent#query-scopes
     *
     * The bitwise operators used below check for the existence or absence of a bit.
     * & is essentially an ==, and & ~ works like a !=
     * See https://www.php.net/manual/en/language.operators.bitwise.php
     */

    /**
     * Return a typical Inbox view, which is read and unread, starred and unstarred
     *  messages that are not archived. Put another way, anything that is not an
     *  Archived message or a Draft.
     * @param Builder $query
     * @return Builder
     */
    public function scopeInbox(Builder $query) : Builder
    {
        return $query->where('NOT status', '&', self::MESSAGE_ARCHIVED)
            ->where('NOT status', '&', self::MESSAGE_DRAFT);
    }

    /**
     * Retrieve unread messages.
     * @param Builder $query
     * @return Builder
     */
    public function scopeUnread(Builder $query) : Builder
    {
        return $query->where('status', '&', self::MESSAGE_READ);
    }

    /**
     * Retrieve draft messages.
     * @param Builder $query
     * @return Builder
     */
    public function scopeDrafts(Builder $query) : Builder
    {
        return $query->where('status', '&', self::MESSAGE_DRAFT);
    }

    /**
     * Retrieve archived messages.
     * @param Builder $query
     * @return Builder
     */
    public function scopeArchive(Builder $query) : Builder
    {
        return $query->where('status', '&', self::MESSAGE_ARCHIVED);
    }

}
