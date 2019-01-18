
{pager jsfunction="WIKIDOT.modules.AWForumModule.listeners.updateList(event,#)" total=$pagerData.totalPages known=$pagerData.knownPages current=$pagerData.currentPage}

{loadmacro set="Forum"}

<div class="thread-container">
	{foreach from=$posts item=post}
		{macro name="forumpost" post=$post linkBackSite=true}
	{/foreach}
</div>