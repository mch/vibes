import streamlit as st
import pandas as pd

data = []

with open("data.txt", "r") as f:
    data = [float(line.strip()) for line in f if line.strip()]

st.write("Hello world")
evens = data[0::2]
odds = data[1::2]
df = pd.DataFrame({
    "Evens": evens,
    "Odds": odds
})
st.write("Evens and Odds")
st.line_chart(df)
