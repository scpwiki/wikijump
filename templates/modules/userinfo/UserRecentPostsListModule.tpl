{loadmacro set="Forum"}


{pager jsfunction="WIKIDOT.modules.UserRecentPostsModule.listeners.updateList" total=$pagerData.totalPages known=$pagerData.knownPages current=$pagerData.currentPage}


<div class="thread-container">
	{foreach from=$posts item=post}
		{macro name="forumpost" post=$post linkBackSite=true}
	{/foreach}
</div>

