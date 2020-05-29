<div id="page-version-info">
		<table>
			<tr>
				<td>
					{t}Revision no.{/t}:
				</td>
				<td>	
					{$revision->getRevisionNumber()}
				</td>
			</tr>
			<tr>
				<td>{t}Date created{/t}:</td>
				<td>
					<span class="odate">{$revision->getDateLastEdited()->getTimestamp()}|%e %b %Y, %H:%M %Z|agohover</span>
				</td>
			</tr>
			<tr>
				<td>{t}By{/t}:</td>
				<td>
					{printuser user=$revision->getUserOrString() image="true"}
				</td>
			</tr>
			<tr>
				<td>
					{t}Page name{/t}:
				</td>
				<td>	
					{$metadata->getUnixName()}
				</td>
			</tr>
		</table>
		{if $revision->getComments() && $revision->getComments()!=null}
		<div style="margin: 4px 0">
			<em>{$revision->getComments()}</em>
		</div>
		{/if}
		<a href="javascript:void(0)" onclick="OZONE.visuals.scrollTo('action-area')">{t}down to versions{/t}</a>
		| <a href="javascript:void(0)" onclick="document.getElementById('page-version-info').style.display='none'">{t}close this box{/t}</a>
	</div>

{$pageContent}

