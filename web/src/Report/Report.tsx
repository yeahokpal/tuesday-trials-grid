import React, { useState } from 'react';

const Report = () => {
  const [selectedOption, setSelectedOption] = useState<string>('Option 1');

  const handleChange = (event: React.ChangeEvent<HTMLSelectElement>) => {
    setSelectedOption(event.target.value);
  };

  return (
    <>
    <div className="container">
        <h2>What issue did you find?</h2>
        <select value={selectedOption} onChange={handleChange}>
          <option value="Characters Played">Characters Played</option>
          <option value="Controller Used">Controller Used</option>
          <option value="Trials Named After">Trials Named After</option>
          <option value="On Stream">On Stream</option>
          <option value="Site Error/Issue">Site Error/Issue</option>
          <option value="Other">Other</option>
        </select>
        <button>Submit</button>
      </div>
    </>
  );
};


export default Report;