<script setup lang="ts">
import { computed, onMounted, type Ref, ref } from 'vue';
import TitleBar from '../components/TitleBar.vue';
import { ApolloClient, gql } from '@apollo/client';
import { HttpLink } from '@apollo/client';
import { InMemoryCache } from '@apollo/client';
import { ServerError } from '@apollo/client';
import { useRouter } from 'vue-router';
import { setAuthProp } from '../auth';
import type { LinkType } from '../api';
import Button from '../components/Button.vue';

const router = useRouter();
const linkPagesCount: Ref<number | null> = ref(null);
const linkPageVisible: Ref<LinkType[] | null> = ref(null);
const linkPageIndex: Ref<number | null> = ref(null);

const formatDate = (timestamp: number) => {
  return new Date(timestamp * 1000).toLocaleDateString();
};

const visiblePages = computed(() => {
  const current = linkPageIndex.value ?? 0;
  const total = linkPagesCount.value ?? 0;
  const visibleCount = 6;

  if (total <= visibleCount) {
    return Array.from({ length: total }, (_, i) => i + 1);
  }

  let start = current - Math.floor(visibleCount / 2);
  let end = current + Math.floor(visibleCount / 2);

  if (start < 1) {
    start = 1;
    end = start + visibleCount - 1;
  }

  if (end > total) {
    end = total;
    start = end - visibleCount + 1;
  }

  return Array.from({ length: end - start + 1 }, (_, i) => start + i);
})

const client = new ApolloClient({
  link: new HttpLink({ uri: "/api" }),
  cache: new InMemoryCache(),
});

onMounted(async () => {
  type Response = {
    fetchLinkPagesCount: {
      totalPages: number
    },
    fetchLinksPages: {
      links: LinkType[]
    }
  };

  try {
    const res = await client.query({
      query: gql`{
        fetchLinkPagesCount {
          totalPages
        }
        fetchLinksPages(page: 1) {
          links {
            id shortLink longLink expires created clicks
          }
        }
      }`,
      fetchPolicy: "network-only"
    });

    const data = res.data as Response;

    linkPagesCount.value = data.fetchLinkPagesCount.totalPages;
    linkPageVisible.value = data.fetchLinksPages.links;
    linkPageIndex.value = 1;
  } catch (err: any) {
    if (ServerError.is(err)) {
      const serverError = err as ServerError;

      if (serverError.statusCode == 401) {
        setAuthProp(null);
        router.push("/login");
      }
    }

    console.error(`api call failed: ${err}`);
  }
});

const goToPage = async (idx: number) => {
  type Response = {
    fetchLinkPagesCount: {
      totalPages: number
    },
    fetchLinksPages: {
      links: LinkType[]
    }
  };

  if (linkPagesCount.value) {
    if (idx >= linkPagesCount.value) {
      idx = linkPagesCount.value;
    } else if (idx <= 1) {
      idx = 1;
    }

    try {
      const res = await client.query({
        query: gql`
          query FetchLinksPages($page: Int!) {
            fetchLinksPages(page: $page) {
              links {
                id shortLink longLink expires created clicks
              }
            }
          }
        `,
        fetchPolicy: "network-only",
        variables: {
          page: idx
        }
      });

      const data = res.data as Response;

      linkPageVisible.value = data.fetchLinksPages.links;
      linkPageIndex.value = idx;
    } catch (err: any) {
      if (ServerError.is(err)) {
        const serverError = err as ServerError;

        if (serverError.statusCode == 401) {
          setAuthProp(null);
          router.push("/login");
        }
      }

      console.error(`api call failed: ${err}`);
    }
  }
};

const deleteLink = async (linkId: number) => {
  if (linkPageVisible.value && linkPageIndex.value && linkPagesCount.value) {
    try {
      await client.mutate({
        mutation: gql`
        mutation deleteLink($linkId: Int!) {
          deleteLink(linkId: $linkId)
        }
      `,
        variables: {
          linkId: linkId
        }
      });

      linkPageVisible.value = linkPageVisible.value.filter(v => {
        return v.id != linkId
      });

      if (linkPageVisible.value.length == 0) {
        type Response = {
          fetchLinkPagesCount: {
            totalPages: number
          },
        };

        const res = await client.query({
          query: gql`{
            fetchLinkPagesCount {
              totalPages
            }
          }`,
          fetchPolicy: "network-only",
        });

        const data = res.data as Response;

        linkPagesCount.value = data.fetchLinkPagesCount.totalPages;

        let nextPage = linkPageIndex.value;

        if (linkPageIndex.value > linkPagesCount.value) {
          nextPage = linkPagesCount.value;
        }

        if (nextPage < 1) {
          nextPage = 1;
        }

        goToPage(nextPage);
      }
    } catch (err: any) {
      if (ServerError.is(err)) {
        const serverError = err as ServerError;

        if (serverError.statusCode == 401) {
          setAuthProp(null);
          router.push("/login");
        }
      }

      console.error(`api call failed: ${err}`);
    }
  }
}
</script>

<template>
  <TitleBar />
  <div class="py-8 flex justify-center">
    <div class="w-[800px] flex flex-col gap-2">
      <div class="flex flex-row">
        <p class="text-lg">Shorten Links</p>
        <div class="flex-auto"></div>
        <button @click="router.push('/dashboard/add')"
          class="bg-green-900 hover:bg-green-800 text-white px-3 py-1 rounded text-sm">
          Add
        </button>
      </div>
      <div v-if="linkPageVisible && linkPageVisible.length > 0" class="w-full">
        <div class="overflow-x-auto">
          <table class="w-full">
            <thead>
              <tr class="bg-blue-950">
                <th class="px-4 py-2 border-b text-left">Short Link</th>
                <th class="px-4 py-2 border-b text-left">Long Link</th>
                <th class="px-4 py-2 border-b text-left">Created</th>
                <th class="px-4 py-2 border-b text-left">Expires</th>
                <th class="px-4 py-2 border-b text-left">Clicks</th>
                <th class="px-4 py-2 border-b text-center">Actions</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="link in linkPageVisible" :key="link.shortLink" class="hover:bg-gray-950">
                <td class=" px-4 py-2 border-b">
                  <a :href="`/dashboard/link/${link.id}`" target="_blank"
                    class="text-blue-400 hover:underline truncate block max-w-xs">
                    {{ link.shortLink }}
                  </a>

                </td>
                <td class="px-4 py-2 border-b">
                  <a :href="link.longLink" target="_blank"
                    class="text-blue-400 hover:underline truncate block max-w-xs">
                    {{ link.longLink }}
                  </a>
                </td>
                <td class="px-4 py-2 border-b">{{ formatDate(link.created) }}</td>
                <td class="px-4 py-2 border-b">{{ link.expires ? formatDate(link.expires) : "-" }}</td>
                <td class="px-4 py-2 border-b">{{ link.clicks }}</td>
                <td class="px-4 py-2 border-b text-center">
                  <button @click="deleteLink(link.id)"
                    class="bg-red-900 hover:bg-red-800 text-white px-3 py-1 rounded text-sm">
                    Delete
                  </button>
                </td>
              </tr>
            </tbody>
          </table>
        </div>

        <div v-if="linkPagesCount && linkPagesCount > 1" class="flex w-full justify-center items-center gap-2 mt-4">
          <Button v-if="linkPageIndex ?? 0 > linkPagesCount" :text="'Prev'" :call="async () => {
            await goToPage((linkPageIndex ?? 1) - 1);
          }" />
          <Button :text="page.toString()" :call="async () => {
            await goToPage(page);
          }" v-for="page in visiblePages" :key="page"
            :class="page === (linkPageIndex ?? 0) ? 'bg-slate-900 text-gray-50' : ''" />
          <Button v-if="linkPageIndex ?? 0 < linkPagesCount" :text="'Next'" :call="async () => {
            await goToPage((linkPageIndex ?? 0) + 1);
          }" />
        </div>
      </div>
      <div v-else>
        <p class="text-center">No links are available.</p>
      </div>
    </div>
  </div>
</template>
