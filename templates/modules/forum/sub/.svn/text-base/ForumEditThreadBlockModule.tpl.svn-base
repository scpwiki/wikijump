<h2>{t}Block posting to this thread{/t}</h2>
<p>
	{t}If enabled - regular users will not be able to post new messages to 
	this thread, but Administrators and Forum Moderators will still be allowed.{/t}
</p>

{*
<p style="text-align: center">
	<input type="checkbox" id="thread-block-checkbox" {if $thread->getBlocked()}checked="checked"{/if}/> thread blocked
	<input type="button" value="save" onclick="WIKIDOT.modules.ForumEditThreadBlockModule.listeners.save(event)"/>
	<input type="button" value="cancel" onclick="$('thread-action-area').style.display='none'"/>
</p>
*}

<form>
	<table class="form">
		<tr>
			<td>
				{t}Thread blocked{/t}:
			</td>
			<td>
				<input class="checkbox" type="checkbox" id="thread-block-checkbox" {if $thread->getBlocked()}checked="checked"{/if}/>
			</td>
		</tr>
	</table>
	<div class="buttons">
		<input type="button" value="{t}cancel{/t}" onclick="$('thread-action-area').style.display='none'"/>
		<input type="button" value="{t}save{/t}" onclick="WIKIDOT.modules.ForumEditThreadBlockModule.listeners.save(event)"/>
		</div>
</form>