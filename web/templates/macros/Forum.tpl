{**
* @param post object
* @param options boolean
* @param reply boolean
* @param revisionOptions boolean
* @param headOptions boolean
* @param linkBack boolean
*}
{defmacro name="forumpost"}
	{if isset($linkBackSite)}
		{assign var=site value=$post->getSite()}
	{/if}
	<div class="post" id="post-{$post->getPostId()}">
		<div class="long">
			<div class="head">
				{if isset($headOptions)}
					<div class="options">
						<a href="javascript:;" onclick="togglePostFold(event,{$post->getPostId()})">{t}fold{/t}</a>
					</div>
				{/if}
				<div class="title" id="post-title-{$post->getPostId()}">
					{if isset($linkBack)}
						{assign var=thread value=$post->getForumThread()}
						<a href="/forum/t-{$thread->getThreadId()}/{$thread->getUnixifiedTitle()}#post-{$post->getPostId()}">{$post->getTitle()|escape}</a>
					{else}
						{if isset($linkBackSite)}
							{assign var=thread value=$post->getForumThread()}
							<a href="{$HTTP_SCHEMA}://{$site->getDomain()}/forum/t-{$thread->getThreadId()}/{$thread->getUnixifiedTitle()}#post-{$post->getPostId()}">{$post->getTitle()|escape}</a>
						{else}
							{$post->getTitle()|escape}
						{/if}
					{/if}
				</div>
				<div class="info">
					{printuser user=$post->getUserOrString() image=true} <span class="odate">{$post->getDatePosted()->getTimestamp()}|%e %b %Y, %H:%M %Z|agohover</span>
					{if isset($linkBack)}
						{assign var=category value=$thread->getForumCategory()}
						<br/>
						{t}in discussion{/t} <a href="/forum/c-{$category->getCategoryId()}/{$category->getUnixifiedName()|escape}">{$category->getForumGroup()->getName()|escape} / {$category->getName()|escape}</a> &raquo;
						<a href="/forum/t-{$thread->getThreadId()}/{$thread->getUnixifiedTitle()}">{$thread->getTitle()|escape}</a>
					{/if}
					{if isset($linkBackSite)}
						<br/>
						on site <a href="{$HTTP_SCHEMA}://{$site->getDomain()}">{$site->getName()|escape}</a><br/>
						in discussion: <a href="{$HTTP_SCHEMA}://{$site->getDomain()}/forum/t-{$thread->getThreadId()}/{$thread->getUnixifiedTitle()}">{$thread->getTitle()|escape}</a>
					{/if}
				</div>
			</div>
			<div class="content" id="post-content-{$post->getPostId()}">
				{$post->getText()}
			</div>
			{if isset($revisionOptions) && $post->getRevisionNumber()>0}
				<div class="changes">
					{t}last edited on{/t} <span class="odate">{$post->getDateLastEdited()->getTimestamp()}|%e %b %Y, %H:%M %Z|agohover</span>
					{t}by{/t} {printuser user=$post->getEditedUserOrString()}
					<a href="javascript:;" onclick="Wikijump.modules.ForumViewThreadModule.listeners.showHistory(event,{$post->getPostId()})">+ {t}show more{/t}</a>
				</div>
				<div class="revisions" style="display: none"></div>
			{/if}

			<div class="options">

				{* put reply or not... *}
				{if  isset($reply)}
					<strong><a href="javascript:;" onclick="postReply(event,{$post->getPostId()})">{t}reply{/t}</a></strong>
				{/if}
				{if isset($reply) && isset($options)}
					|
				{/if}
				{if isset($options)}
					<a href="javascript:;" onclick="togglePostOptions(event,{$post->getPostId()})">{t}options{/t}</a>
				{/if}
			</div>

			<div id="post-options-{$post->getPostId()}" class="options" style="display: none">
			</div>
		</div>
		<div class="short">
			{if isset($headOptions)}
				<a class="options" href="javascript:;" onclick="togglePostFold(event,{$post->getPostId()})">{t}unfold{/t}</a>
			{/if}
			<a class="title" href="javascript:;" {if isset($headOptions)}onclick="togglePostFold(event,{$post->getPostId()})"{/if}>{$post->getTitle()|escape}</a> {t}by{/t} {printuser user=$post->getUserOrString() image=true}, <span class="odate">{$post->getDatePosted()->getTimestamp()}|%e %b %Y, %H:%M %Z|agohover</span>
		</div>
	</div>
{/defmacro}
