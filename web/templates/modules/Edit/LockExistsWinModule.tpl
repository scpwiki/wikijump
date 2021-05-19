
<div class="owindow error" style="width: 80%;">
	<div class="title">
		{t}Page locked{/t}
	</div>
	<div class="content">
		<h1>{t}Page lock conflict{/t}</h1>
		<p>
			{t}The page is currently locked by another user. The details follow:{/t}
		</p>
		{if isset($yourself)}
			<p>
			{t}It seems that you already possess the edit lock for this page. However it is not possible to
			open edit form because current situation suggests there might be another window opened by
			you where the page is edited or you have not exited previous edit properly.{/t}
			</p>
		{/if}
		{foreach from=$locks item=lock}
		<p>
			{t}Locked by{/t}: {printuser user=$lock->getUserOrString() image="true"}<br/>
			{t}Lock mode{/t}: {$lock->getMode()|escape}<br/>
			{t}Started editing{/t}: <span class="odate">{$lock->getDateStarted()->getTimestamp()}</span> ({$lock->getStartedAgo()} {t}seconds ago{/t})<br/>
			{t}Lock will expire in{/t}: {$lock->getExpireIn()} {t}seconds (if user remains inactive){/t}</br>
		</p>
		{/foreach}
		<p>
			{t escape=no}If you are <strong>absolutely</strong> sure that the lock is not valid, you can force
			the lock removal manually pressing the button below.{/t}
		</p>
		<div class="button-bar">
			<a href="javascript:;" onclick="OZONE.dialog.cleanAll()">{t}cancel{/t}</a>
			<a id="edit-recheck-lock" href="javascript:;" onclick="Wikijump.page.listeners.editClick(event)">{t}check again{/t}</a>
			<a id="edit-force-remove-lock" href="javascript:;" onclick="Wikijump.modules.PageEditModule.listeners.forcePageEditLockRemove(event)">{t}force lock removal{/t}</a>
		</div>
	</div>
</div>
