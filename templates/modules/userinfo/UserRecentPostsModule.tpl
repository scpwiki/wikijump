<h1>{t}Recent posts and comments{/t}</h1>

<div class="forum-recent-posts-box" >

	{*<input type="hidden" id="recent-posts-user-id" value="{$userId}"/>*}

	<div id="forum-recent-posts-list">
		{module name="userinfo/UserRecentPostsListModule" userId=$userId}
	</div>

</div>