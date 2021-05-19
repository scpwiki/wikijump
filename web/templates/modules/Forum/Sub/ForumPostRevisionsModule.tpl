<a href="javascript:;" onclick="Wikijump.modules.ForumViewThreadModule.listeners.hideHistory(event, {$post->getPostId()})">- {t}hide{/t}</a>

<div class="title">{t}Post revisions{/t}</div>
{assign var=first value=true}
<table>
	{foreach from=$revisions item=revision}
		<tr {if isset($first)}class="active"{assign var=first value=false}{/if}>
			<td>
				{printuser user=$revision->getUserOrString()}
			</td>
			<td>
				<span class="odate">{$revision->getDate()->getTimestamp()}|%e %b %Y, %H:%M %Z|agohover</span>
			</td>
			<td>
				| <a href="javascript:;" onclick="Wikijump.modules.ForumViewThreadModule.listeners.showRevision(event, {$revision->getRevisionId()})">{t}show revision{/t}</a>
			</td>
		</tr>
	{/foreach}
</table>
