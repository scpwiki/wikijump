<div id="delete-post-{$post->getPostId()}" class="action-area">
	<h1>{t}Delete forum post{/t}</h1>

	<p>
		{t}Are you sure you want to delete the above post? There is no way to recover deleted posts.{/t}
	</p>
	{if isset($hasChildren)}
		<p>
			{t}The post you want to delete is placed in a nested environment. All other
			posts that depend (are lower in a dep tree) are also surrounded by a red frame and
			will also be deleted. <strong>In most cases it is better to edit the post instead of
			deleting it.</strong>{/t}
		</p>
	{/if}
	<form>
		<div class="buttons">
			<input type="button" value="{t}cancel{/t}" onclick="Wikijump.modules.ForumDeletePostModule.listeners.cancel(event, {$post->getPostId()})"/>
			<input type="button" value="{t}delete{/t}!" onclick="Wikijump.modules.ForumDeletePostModule.listeners.deletePost(event, {$post->getPostId()})"/>
		</div>
	</form>
</div>
