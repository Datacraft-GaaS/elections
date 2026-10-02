/**
 * Copyright (c) 2026 Nyimbi Odero
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

// Minimal message catalogue for the hr-public portal.
// `en` is complete; `sw` covers the chrome, hero, steps, form labels and key
// states. Any key missing in a locale falls back to English.

export const messages = {
  en: {
    'nav.apply': 'Apply',
    'nav.status': 'Check status',
    'nav.home': 'Poll workers',
    'brand': 'Poll Worker Portal',
    'brand.short': 'Poll workers',
    'hero.eyebrow': 'Serving your county on election day',
    'hero.title': 'Do the day your community counts on.',
    'hero.lead':
      'Become a Presiding Officer, Deputy Presiding Officer or Polling Clerk. You will run your polling station, protect the secrecy of the vote, and be paid for a full, accountable day of service.',
    'hero.cta': 'Start your application',
    'hero.cta.status': 'Already applied? Check your status',
    'hero.note': 'Free to apply · Training provided · No experience required',

    'roles.eyebrow': 'Three ways to serve',
    'roles.title': 'Choose where you fit',
    'roles.po.title': 'Presiding Officer',
    'roles.po.body':
      'You lead the polling station: open and close the poll, certify the count, and sign the declaration of results.',
    'roles.dpo.title': 'Deputy Presiding Officer',
    'roles.dpo.body':
      'You support the Presiding Officer and take charge in their absence, keeping the register and the seal.',
    'roles.clerk.title': 'Polling Clerk',
    'roles.clerk.body':
      'You check voters in against the register, issue ballot papers, and keep the queue moving fairly.',

    'how.eyebrow': 'How it works',
    'how.title': 'From application to polling day',
    'how.steps': [
      { title: 'Apply online', body: 'Tell us about yourself and upload your papers in one sitting.' },
      { title: 'We verify your documents', body: 'Your ID, PIN and police clearance are checked against the records.' },
      { title: 'Train and be assessed', body: 'Take the competency exam and, if needed, a short interview.' },
      { title: 'You are posted', body: 'If selected, you are assigned a station and notified in the app.' },
      { title: 'Serve and be paid', body: 'Run your station, complete your duties, and get paid through the app.' },
    ],

    'need.eyebrow': 'What you will need',
    'need.title': 'Gather these before you start',
    'need.items': [
      'National ID card (both sides)',
      'KRA PIN certificate',
      'Police clearance certificate (issued within 12 months)',
      'Academic or professional certificates',
      'A recent passport photograph',
      'Bank confirmation letter (for payment)',
    ],

    'cta.title': 'Ready when you are',
    'cta.body': 'Applications take about ten minutes. You can save and return.',
    'cta.button': 'Apply to be a poll worker',

    'footer.line': 'An independent, public service. Free and open to every eligible citizen.',
    'footer.help': 'Need help? Call the support line',
    'footer.privacy': 'Privacy',
    'footer.access': 'Accessibility',
    'footer.contact': 'Contact',

    // apply
    'apply.title': 'Application to serve',
    'apply.lead': 'A few short steps. We never share your documents beyond verification.',
    'apply.step.you': 'About you',
    'apply.step.role': 'Your role',
    'apply.step.papers': 'Your papers',
    'apply.step.oath': 'Your oath',
    'apply.step.seal': 'Review & seal',

    'apply.you.title': 'Tell us about yourself',
    'apply.name': 'Full legal name',
    'apply.name.hint': 'As it appears on your national ID.',
    'apply.id': 'National ID number',
    'apply.id.hint': 'We store only a fingerprint of this number, never the number itself.',
    'apply.county': 'County of residence',
    'apply.phone': 'Phone number',
    'apply.phone.hint': 'We will text you updates. WhatsApp and voice supported.',
    'apply.email': 'Email address (optional)',

    'apply.role.title': 'Choose your role',
    'apply.role.county': 'County where you would like to serve',
    'apply.role.any': 'Any county',

    'apply.papers.title': 'Upload your papers',
    'apply.papers.lead': 'Add one file per item. Photos from your phone are fine.',
    'apply.papers.add': 'Choose file',
    'apply.papers.uploaded': 'Uploaded',
    'apply.papers.required': 'Required',
    'apply.papers.optional': 'Optional',

    'apply.oath.title': 'Your declaration',
    'apply.oath.body':
      'I declare that the information and documents I have provided are true and complete. If selected, I will uphold the secrecy of the vote and perform my duties honestly and impartially.',
    'apply.oath.confirm': 'I make this declaration truthfully.',

    'apply.seal.title': 'Review and seal your application',
    'apply.seal.review': 'Check that everything is correct. You can go back and change any step.',
    'apply.back': 'Back',
    'apply.continue': 'Continue',
    'apply.review': 'Review',
    'apply.submit': 'Seal & submit',

    'apply.done.eyebrow': 'Application received',
    'apply.done.title': 'You are in the ledger.',
    'apply.done.body':
      'Keep this reference number. We will text you as your application moves through screening, document verification, training and posting.',
    'apply.done.next': 'What happens next',
    'apply.done.steps': [
      'We verify your documents against official records.',
      'You are invited to training and the competency exam.',
      'If selected, you receive your posting and instructions in the app.',
    ],
    'apply.done.track': 'Track your application',

    // status
    'status.title': 'Check your application',
    'status.lead': 'Enter the reference number from your confirmation.',
    'status.ref': 'Reference number',
    'status.lookup': 'Look up status',
    'status.stages': ['Applied', 'Documents', 'Examined', 'Selected', 'Appointed'],

    // validation
    'err.required': 'This field is required.',
    'err.id': 'Enter a valid national ID number.',
    'err.phone': 'Enter a valid phone number.',
    'err.oath': 'You must make the declaration to continue.',
    'err.docs': 'Please upload the required documents to continue.',

    'lang.switch': 'Kiswahili',
  },

  sw: {
    'nav.apply': 'Omba',
    'nav.status': 'Angalia hali',
    'nav.home': 'Wafanyakazi wa uchaguzi',
    'brand': 'Mlango wa Wafanyakazi',
    'brand.short': 'Wafanyakazi',
    'hero.eyebrow': 'Kutumikia kaunti yako siku ya uchaguzi',
    'hero.title': 'Fanya siku ambayo jamii yako inakutegemea.',
    'hero.lead':
      'Kuwa Afisa Msimamizi, Naibu Afisa Msimamizi au Karani wa Uchaguzi. Utaendesha kituo chako cha kupigia kura, kulinda siri ya kura, na kulipwa kwa siku kamili ya huduma.',
    'hero.cta': 'Anza ombi lako',
    'hero.cta.status': 'Tayari ulituma? Angalia hali',
    'hero.note': 'Bure kuomba · Mafunzo hutolewa · Hakuna uzoefu unaohitajika',

    'roles.eyebrow': 'Njia tatu za kutumikia',
    'roles.title': 'Chagua unapofaa',
    'roles.po.title': 'Afisa Msimamizi',
    'roles.po.body':
      'Wewe unaongoza kituo cha kupigia kura: fungua na funga uchaguzi, thibitisha hesabu, na utie sahihi tangazo la matokeo.',
    'roles.dpo.title': 'Naibu Afisa Msimamizi',
    'roles.dpo.body':
      'Unamsaidia Afisa Msimamizi na kuchukua nafasi yake akiwa hayupo, ukilinda daftari na muhuri.',
    'roles.clerk.title': 'Karani wa Uchaguzi',
    'roles.clerk.body':
      'Unawahakiki wapigakura dhidi ya daftari, kutoa karatasi za kura, na kuhakikisha foleni inaendelea kwa haki.',

    'how.eyebrow': 'Jinsi inavyofanya kazi',
    'how.title': 'Kutoka kwa ombi hadi siku ya uchaguzi',
    'how.steps': [
      { title: 'Omba mtandaoni', body: 'Tuambie kukuhusu na pakia nyaraka zako kwa mkupuo mmoja.' },
      { title: 'Tunahakiki nyaraka zako', body: 'Kitambulisho, PIN na cheti cha polisi hulinganishwa na kumbukumbu.' },
      { title: 'Pata mafunzo na tathmini', body: 'Fanya mtihani wa ustadi na, ikihitajika, mahojiano mafupi.' },
      { title: 'Unatengewa kituo', body: 'Ukichaguliwa, unapewa kituo na kutaarifiwa kwenye programu.' },
      { title: 'Huduma na malipo', body: 'Endesha kituo chako, kamilisha majukumu, na ulipwe kupitia programu.' },
    ],

    'need.eyebrow': 'Unachohitaji',
    'need.title': 'Kusanya hivi kabla ya kuanza',
    'need.items': [
      'Kitambulisho cha kitaifa (pande zote mbili)',
      'Cheti cha PIN ya KRA',
      'Cheti cha usafi wa polisi (kilichotolewa ndani ya miezi 12)',
      'Vyeti vya taaluma au kitaaluma',
      'Picha ya pasipoti ya hivi karibuni',
      'Barua ya uthibitisho wa benki (kwa malipo)',
    ],

    'cta.title': 'Tayari wakati wowote',
    'cta.body': 'Ombi huchukua takriban dakika kumi. Unaweza kuhifadhi na kurudi.',
    'cta.button': 'Omba kuwa mfanyakazi wa uchaguzi',

    'footer.line': 'Huduma huru ya umma. Bure na wazi kwa kila raia anayestahiki.',
    'footer.help': 'Unahitaji msaada? Piga laini ya usaidizi',
    'footer.privacy': 'Faragha',
    'footer.access': 'Ufikivu',
    'footer.contact': 'Wasiliana nasi',

    'apply.title': 'Ombi la kutumikia',
    'apply.lead': 'Hatua chache fupi. Hatushiriki nyaraka zako zaidi ya uhakiki.',
    'apply.step.you': 'Kukuhusu',
    'apply.step.role': 'Nafasi yako',
    'apply.step.papers': 'Nyaraka zako',
    'apply.step.oath': 'Kiapo chako',
    'apply.step.seal': 'Kagua na muhuri',

    'apply.you.title': 'Tuambie kukuhusu',
    'apply.name': 'Jina kamili la kisheria',
    'apply.name.hint': 'Kama lilivyo kwenye kitambulisho chako cha kitaifa.',
    'apply.id': 'Nambari ya kitambulisho',
    'apply.id.hint': 'Tunahifadhi tu alama ya nambari hii, si nambari yenyewe.',
    'apply.county': 'Kaunti ya makaazi',
    'apply.phone': 'Nambari ya simu',
    'apply.phone.hint': 'Tutakutumia taarifa. WhatsApp na sauti zinaungwa mkono.',
    'apply.email': 'Barua pepe (si lazima)',

    'apply.role.title': 'Chagua nafasi yako',
    'apply.role.county': 'Kaunti ungependa kutumikia',
    'apply.role.any': 'Kaunti yoyote',

    'apply.papers.title': 'Pakia nyaraka zako',
    'apply.papers.lead': 'Ongeza faili moja kwa kila kipengee. Picha kutoka simu yako zinatosha.',
    'apply.papers.add': 'Chagua faili',
    'apply.papers.uploaded': 'Imepakiwa',
    'apply.papers.required': 'Inahitajika',
    'apply.papers.optional': 'Si lazima',

    'apply.oath.title': 'Tamko lako',
    'apply.oath.body':
      'Natangaza kwamba taarifa na nyaraka nilizotoa ni za kweli na kamili. Nikichaguliwa, nitalinda siri ya kura na kutekeleza majukumu yangu kwa uaminifu na bila upendeleo.',
    'apply.oath.confirm': 'Nafanya tamko hili kwa ukweli.',

    'apply.seal.title': 'Kagua na weka muhuri kwenye ombi lako',
    'apply.seal.review': 'Hakikisha kila kitu ni sahihi. Unaweza kurudi na kubadilisha hatua yoyote.',
    'apply.back': 'Rudi',
    'apply.continue': 'Endelea',
    'apply.review': 'Kagua',
    'apply.submit': 'Weka muhuri na tuma',

    'apply.done.eyebrow': 'Ombi limepokelewa',
    'apply.done.title': 'Umeandikwa kwenye daftari.',
    'apply.done.body':
      'Hifadhi nambari hii ya kumbukumbu. Tutakutumia ujumbe kadri ombi lako linavyosonga kupitia uhakiki, uhakiki wa nyaraka, mafunzo na utengeji.',
    'apply.done.next': 'Kinachofuata',
    'apply.done.steps': [
      'Tunahakiki nyaraka zako dhidi ya kumbukumbu rasmi.',
      'Unaalikwa kwenye mafunzo na mtihani wa ustadi.',
      'Ukichaguliwa, unapata kituo chako na maelekezo kwenye programu.',
    ],
    'apply.done.track': 'Fuatilia ombi lako',

    'status.title': 'Angalia ombi lako',
    'status.lead': 'Weka nambari ya kumbukumbu kutoka kwa uthibitisho wako.',
    'status.ref': 'Nambari ya kumbukumbu',
    'status.lookup': 'Angalia hali',
    'status.stages': ['Imepokelewa', 'Nyaraka', 'Imepimwa', 'Imechaguliwa', 'Imeteuliwa'],

    'err.required': 'Sehemu hii inahitajika.',
    'err.id': 'Weka nambari sahihi ya kitambulisho.',
    'err.phone': 'Weka nambari sahihi ya simu.',
    'err.oath': 'Lazima ufanye tamko ili kuendelea.',
    'err.docs': 'Tafadhali pakia nyaraka zinazohitajika kuendelea.',

    'lang.switch': 'English',
  },
};

export function t(locale, key) {
  const dict = messages[locale] || messages.en;
  return dict[key] ?? messages.en[key] ?? key;
}
