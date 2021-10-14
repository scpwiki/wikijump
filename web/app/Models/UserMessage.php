<?php
declare(strict_types=1);

namespace Wikijump\Models;

use Illuminate\Auth\Access\AuthorizationException;
use Illuminate\Database\Eloquent\Builder;
use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Relations\BelongsTo;
use Illuminate\Support\Facades\DB;
use Illuminate\Support\Facades\Gate;
use Wikijump\Traits\UsesBitmasks;
use Wikijump\Traits\UsesUUIDs;

/**
 * Private (user-to-user) Messages
 * @package Wikijump\Models
 */
class UserMessage extends Model
{
    use HasFactory;
    use UsesBitmasks;
    use UsesUUIDs;

    /** Various flags to set on a PM. */
    public const MESSAGE_READ = 1;
    public const MESSAGE_DRAFT = 2;
    public const MESSAGE_STARRED = 4;
    public const MESSAGE_ARCHIVED = 8;
    public const MESSAGE_SENT = 16;
    # const MESSAGE_RESERVED = 32;
    # const MESSAGE_RESERVED = 64;
    # const MESSAGE_RESERVED = 128;

    /**
     * ORM for message sender.
     * @return BelongsTo
     */
    public function sender(): BelongsTo
    {
        return $this->belongsTo(User::class, 'from_user_id');
    }

    /**
     * ORM for message recipient.
     * @return BelongsTo
     */
    public function recipient(): BelongsTo
    {
        return $this->belongsTo(User::class, 'to_user_id');
    }

    /**
     * Saves the model and also saves a copy for the user's Sent folder.
     * @throws AuthorizationException
     */
    public function send()
    {
        $permission = Gate::inspect('send', $this);
        if ($permission->denied()) {
            throw new AuthorizationException($permission->message());
        }

        return DB::transaction(function () {
            $this->save();

            $sent = $this->replicate();
            $sent->setFlag(UserMessage::MESSAGE_SENT);
            $sent->save();
        });
    }

    /**
     * Generate a preview of a message.
     * @param int $length
     * @return string
     */
    public function preview(int $length = 200): string
    {
        $preview = substr(strip_tags($this->body), 0, $length);
        if (strlen($preview) >= $length) {
            $preview = preg_replace('/\w+$/', '', $preview) . '...';
        }
        return $preview;
    }

    /**
     * Mark a message as read.
     * @return bool
     */
    public function markAsRead(): bool
    {
        $this->setFlag(self::MESSAGE_READ);
        return $this->save();
    }

    /**
     * Mark a message as unread.
     * @return bool
     */
    public function markAsUnread(): bool
    {
        $this->clearFlag(self::MESSAGE_READ);
        return $this->save();
    }

    /**
     * Mark a message as a draft.
     * The `to_user_id` field is nullable to allow for this.
     * @return bool
     */
    public function markAsDraft(): bool
    {
        $this->setFlag(self::MESSAGE_DRAFT);
        return $this->save();
    }

    /**
     * Unmark a message as a draft.
     * In practice, this is the point at which a message is "sent" and viewable
     *  to the other party, and visible in a user's "Sent" items instead of an "outbox."
     * @return bool
     */
    public function unmarkAsDraft(): bool
    {
        $this->clearFlag(self::MESSAGE_DRAFT);
        return $this->save();
    }

    /**
     * Star/favorite a message.
     * @return bool
     */
    public function starMessage(): bool
    {
        $this->setFlag(self::MESSAGE_STARRED);
        return $this->save();
    }

    /**
     * Unstar/unfavorite a message.
     * @return bool
     */
    public function unstarMessage(): bool
    {
        $this->clearFlag(self::MESSAGE_STARRED);
        return $this->save();
    }

    /**
     * Archive a message, to get it out of the inbox without deleting it.
     * @return bool
     */
    public function archiveMessage(): bool
    {
        $this->setFlag(self::MESSAGE_ARCHIVED);
        return $this->save();
    }

    /**
     * Move a message from the archive back to the inbox.
     * @return bool
     */
    public function unarchiveMessage(): bool
    {
        $this->clearFlag(self::MESSAGE_ARCHIVED);
        return $this->save();
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
     * & is essentially an ==, but the inverse is harder to cleanly find.
     * Instead we use some raw SQL to work like a != operator.
     * See https://www.php.net/manual/en/language.operators.bitwise.php
     */

    /**
     * Return a typical Inbox view, which is read and unread, starred and unstarred
     *  messages that are not archived. Put another way, anything that is not an
     *  Archived message, a Draft, or a sent item.
     * @param Builder $query
     * @param User $user
     * @return Builder
     */
    public function scopeInbox(Builder $query, User $user): Builder
    {
        return $query
            ->where('to_user_id', $user->id)

            ->whereRaw(self::flagIsNotSet(self::MESSAGE_ARCHIVED))
            ->whereRaw(self::flagIsNotSet(self::MESSAGE_SENT))
            ->whereRaw(self::flagIsNotSet(self::MESSAGE_DRAFT));
    }

    /**
     * Retrieve unread messages.
     * @param Builder $query
     * @param User $user
     * @return Builder
     */
    public function scopeUnread(Builder $query, User $user): Builder
    {
        return $query
            ->where('to_user_id', $user->id)
            ->whereRaw(self::flagIsNotSet(self::MESSAGE_READ));
    }

    /**
     * Retrieve draft messages.
     * @param Builder $query
     * @param User $user
     * @return Builder
     */
    public function scopeDrafts(Builder $query, User $user): Builder
    {
        return $query
            ->where('from_user_id', $user->id)
            ->whereRaw(self::flagIsSet(self::MESSAGE_DRAFT));
    }

    /**
     * Retrieve archived messages.
     * @param Builder $query
     * @param User $user
     * @return Builder
     */
    public function scopeArchive(Builder $query, User $user): Builder
    {
        return $query
            ->where('to_user_id', $user->id)
            ->whereRaw(self::flagIsSet(self::MESSAGE_ARCHIVED));
    }

    /**
     * Retrieve starred messages.
     * @param Builder $query
     * @param User $user
     * @return Builder
     */
    public function scopeStarred(Builder $query, User $user): Builder
    {
        return $query
            ->where('to_user_id', $user->id)
            ->whereRaw(self::flagIsSet(self::MESSAGE_STARRED));
    }

    /**
     * Retrieve sent messages. The only distinction is a MESSAGE_SENT flag.
     * @param Builder $query
     * @param User $user
     * @return Builder
     */
    public function scopeSent(Builder $query, User $user): Builder
    {
        return $query
            ->where('from_user_id', $user->id)
            ->whereRaw(self::flagIsSet(self::MESSAGE_SENT));
    }

    /**
     * Checks for the presence or absence of a particular flag.
     */

    /**
     * Is this message read?
     * @return bool
     */
    public function isRead(): bool
    {
        return $this->getFlag(UserMessage::MESSAGE_READ);
    }

    /**
     * Is this message unread?
     * @return bool
     */
    public function isUnread(): bool
    {
        return !$this->getFlag(UserMessage::MESSAGE_READ);
    }

    /**
     * Is this message a draft?
     * @return bool
     */
    public function isDraft(): bool
    {
        return $this->getFlag(UserMessage::MESSAGE_DRAFT);
    }

    /**
     * Is this message not a draft?
     * @return bool
     */
    public function isNotDraft(): bool
    {
        return !$this->getFlag(UserMessage::MESSAGE_DRAFT);
    }

    /**
     * Is this message starred?
     * @return bool
     */
    public function isStarred(): bool
    {
        return $this->getFlag(UserMessage::MESSAGE_STARRED);
    }

    /**
     * Is this message unstarred?
     * @return bool
     */
    public function isUnstarred(): bool
    {
        return !$this->getFlag(UserMessage::MESSAGE_STARRED);
    }

    /**
     * Is this message archived?
     * @return bool
     */
    public function isArchived(): bool
    {
        return $this->getFlag(UserMessage::MESSAGE_ARCHIVED);
    }

    /**
     * Is this message not archived?
     * @return bool
     */
    public function isNotArchived(): bool
    {
        return !$this->getFlag(UserMessage::MESSAGE_STARRED);
    }

    /**
     * Is this message a sent message?
     * @return bool
     */
    public function isSent(): bool
    {
        return $this->getFlag(UserMessage::MESSAGE_SENT);
    }

    /**
     * Is this message not a sent message?
     * @return bool
     */
    public function isNotSent(): bool
    {
        return !$this->getFlag(UserMessage::MESSAGE_SENT);
    }
}
