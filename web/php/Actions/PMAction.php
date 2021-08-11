<?php
declare(strict_types=1);

namespace Wikidot\Actions;

use Illuminate\Support\Facades\Gate;
use Ozone\Framework\JSONService;
use Ozone\Framework\RunData;
use Ozone\Framework\SmartyAction;
use Wikidot\Utils\NotificationMaker;
use Wikidot\Utils\ProcessException;
use Wikidot\Utils\WDPermissionException;
use Wikijump\Models\User;
use Wikijump\Models\UserMessage;
use Wikijump\Policies\UserPolicy;
use Wikijump\Services\Wikitext\WikitextBackend;

/**
 * Action class for User Message events.
 * @package Wikidot\Actions
 */
class PMAction extends SmartyAction
{

    /**
     * Is the user logged in?
     * @param $runData
     * @return bool
     * @throws WDPermissionException
     */
    public function isAllowed($runData): bool
    {
        if ($runData->getUserId() === null) {
            throw new WDPermissionException(_('You should be logged in in order to send messages.'));
        }
        return true;
    }

    /**
     * Stub class to obey contract.
     * @param RunData $runData
     */
    public function perform($runData)
    {
    }

    /**
     * Validate, authorize, and send a message.
     * @param RunData $runData
     * @throws ProcessException
     * @throws WDPermissionException
     */
    public function sendEvent(RunData $runData)
    {
        $toUser = User::find($runData->get('to_user_id'));
        $subject = $runData->get('subject');
        $body = $runData->get('source');

        /** Validation: */

        if ($toUser === null) {
            $message = __('The recipient does not exist.');
            throw new ProcessException($message, 'no_recipient');
        }

        /**
         * Authorization
         * @see UserPolicy
         */
        $permission = Gate::inspect('message', $toUser);
        if($permission->denied())
        {
            throw new WDPermissionException($permission->message());
        }

        // compile content
        $wt = WikitextBackend::make(PageRenderMode::DIRECT_MESSAGE, null);
        $body = $wt->renderHtml($body)->body;

        $message = new UserMessage([
            'from_user_id' => $runData->id(),
            'to_user_id' => $toUser->id,
            'subject' => $subject,
            'body' => $body
        ]);

        $message->send();

        NotificationMaker::instance()->privateMessageNotification($message);
    }

    /**
     * Save a draft.
     * @param RunData $runData
     */
    public function saveDraftEvent(RunData $runData)
    {
        $body = $runData->get('source');
        $subject = $runData->get('subject');
        $toUserId = $runData->get('to_user_id');

        $message = new UserMessage(
            [
                'from_user_id' => $runData->id(),
                'to_user_id' => $toUserId,
                'subject' => $subject,
                'body' => $body,
                'flags' => UserMessage::MESSAGE_DRAFT
            ]
        );
        $message->save();
    }

    /**
     * Remove selected items from the user's inbox.
     * @param RunData $runData
     */
    public function removeSelectedInboxEvent(RunData $runData)
    {
        $selected = $runData->get('selected');
        $messages = (new JSONService(SERVICES_JSON_LOOSE_TYPE))->decode($selected);

        UserMessage::inbox($runData->user())
            ->whereIn('id', $messages)
            ->delete();
    }

    /**
     * Remove a single item from the inbox and show the next one.
     * @param RunData $runData
     */
    public function removeInboxMessageEvent(RunData $runData)
    {
        $message = UserMessage::find($runData->get('message_id'));

        $nextMessage = UserMessage::inbox($runData->user())
            ->whereTime('created_at', '<', $message->created_at)->first()
            ??
            UserMessage::inbox($runData->user())
            ->whereTime('created_at', '>', $message->created_at)->first();

        $message->delete();

        if ($nextMessage) {
            $runData->ajaxResponseAdd('messageId', $nextMessage->id);
        }
    }

    /**
     * Remove a single message from the Sent folder and display the next one.
     * @param RunData $runData
     */
    public function removeSentMessageEvent(RunData $runData)
    {
        $message = UserMessage::find($runData->get('message_id'));

        $nextMessage = UserMessage::sent($runData->user())
                ->whereTime('created_at', '<', $message->created_at)->first()
            ??
            UserMessage::sent($runData->user())
                ->whereTime('created_at', '>', $message->created_at)->first();

        $message->delete();

        if ($nextMessage) {
            $runData->ajaxResponseAdd('messageId', $nextMessage->id);
        }
    }

    /**
     * Remove selected items from the user's Sent folder.
     * @param RunData $runData
     */
    public function removeSelectedSentEvent(RunData $runData)
    {
        $selected = $runData->get('selected');
        $messages = (new JSONService(SERVICES_JSON_LOOSE_TYPE))->decode($selected);

        UserMessage::sent($runData->user())
            ->whereIn('id', $messages)
            ->delete();
    }

    /**
     * Delete a single draft and present the next one to the user.
     * @param RunData $runData
     */
    public function removeDraftsMessageEvent(RunData $runData)
    {
        $message = UserMessage::find($runData->get('message_id'));

        $nextMessage = UserMessage::drafts($runData->user())
                ->whereTime('created_at', '<', $message->created_at)->first()
            ??
            UserMessage::drafts($runData->user())
                ->whereTime('created_at', '>', $message->created_at)->first();

        $message->delete();

        if ($nextMessage) {
            $runData->ajaxResponseAdd('messageId', $nextMessage->id);
        }
    }

    /**
     * Remove multiple drafts from the user's folder.
     * @param RunData $runData
     */
    public function removeSelectedDraftsEvent(RunData $runData)
    {
        $selected = $runData->get('selected');
        $messages = (new JSONService(SERVICES_JSON_LOOSE_TYPE))->decode($selected);

        UserMessage::drafts($runData->user())
            ->whereIn('id', $messages)
            ->delete();
    }
}
