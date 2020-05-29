<h2>{t}Edit title &amp; description{/t}</h2>
<p>
	{t}To change the title and/or description of this thread please simply fill the form below
	and submit it. Nothing easier.{/t}
</p>

<div class="error-block" id="thread-meta-errors" style="display: none"></div>
<form onsubmit="return false" id="thread-meta-form">
	<input type="hidden" name="threadId" value="{$thread->getThreadId()}"/>
	<table class="form">
		<tr>
			<td>
				{t}Thread title{/t}:
			</td>
			<td>
				<input class="text" type="text" name="title" value="{$thread->getTitle()|escape}" size="50" maxlength="99" />
			</td>
		</tr>
		<tr>
			<td>
				{t}Summary{/t}:
			</td>
			<td>
				<textarea cols="50" rows="2" id="thread-description" name="description">{$thread->getDescription()}</textarea>
				<div class="sub">
					(<span id="desc-charleft"></span> {t}characters left{/t})
				</div>
			</td>
		</tr>
	</table>
	<div class="buttons">
		<input type="button" value="{t}cancel{/t}" onclick="$('thread-action-area').style.display='none'"/>
		<input type="button" value="{t}save{/t}" onclick="WIKIDOT.modules.ForumEditThreadMetaModule.listeners.save(event)"/>
	</div>

</form>